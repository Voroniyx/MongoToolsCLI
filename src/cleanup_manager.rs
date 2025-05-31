use crate::config::Config;
use crate::log::Log;
use crate::utils::Utils;
use chrono::{Duration, NaiveDateTime, Utc};
use glob::glob;
use regex::Regex;
use std::error::Error;
use std::path::PathBuf;

pub struct CleanUpManager {
    pub config: Config,
}

impl CleanUpManager {
    // checks the files in the directory and deletes the oldest ones
    // so that there are only so many backups in the directory that are not older than X(d|w|m|y)
    pub async fn by_max_age(&self, path: Option<String>) {
        let file_result = self.get_oldest_files_by_max_age(
            format!(
                "{}/*.tar.gz",
                Utils::get_project_root_path(path.unwrap().to_string().as_str())
                    .display()
                    .to_string()
                    .as_str()
            )
            .as_str(),
        );

        self.handle_file_result(file_result).await;
    }

    // checks the files in the directory and delete the oldest ones so there are only
    // as many backups in the directory as max_concurrent_backups allows
    pub async fn by_max_concurrent_backups(&self, path: Option<String>) {
        let file_result = self.get_oldest_files_by_date_in_name(
            format!(
                "{}/*.tar.gz",
                Utils::get_project_root_path(path.unwrap().to_string().as_str())
                    .display()
                    .to_string()
                    .as_str()
            )
            .as_str(),
        );

        self.handle_file_result(file_result).await;
    }

    async fn handle_file_result(&self, file_result: Result<Vec<PathBuf>, Box<dyn Error>>) {
        match file_result {
            Ok(files) => {
                for file in files {
                    let delete_result = Utils::delete_file(&file).await;
                    match delete_result {
                        Ok(_) => {
                            Log::success(
                                format!("Deleted file: {}", file.display().to_string())
                                    .as_str(),
                            )
                        }
                        Err(_) => {
                            Log::error(
                                format!("Could not delete file {}", file.display().to_string())
                                    .as_str(),
                            );
                        }
                    }
                }
            }
            Err(e) => {
                Log::error(e.to_string().as_str());
            }
        }
    }

    fn get_oldest_files_by_date_in_name(
        &self,
        glob_pattern: &str,
    ) -> Result<Vec<PathBuf>, Box<dyn Error>> {
        let re = Regex::new(r"(\d{4}-\d{2}-\d{2}_\d{2}-\d{2}-\d{2})")?;
        let max_concurrent_backups = self.config.max_concurrent_backups.clone();

        let mut files_with_timestamps: Vec<(PathBuf, u64)> = glob(glob_pattern)?
            .filter_map(Result::ok)
            .filter_map(|path| {
                self.extract_numeric_timestamp_from_filename(&path, &re)
                    .map(|ts| (path, ts))
            })
            .collect();

        files_with_timestamps.sort_by_key(|(_, ts)| *ts);

        let oldest = files_with_timestamps
            .into_iter()
            .take(max_concurrent_backups)
            .map(|(path, _)| path)
            .collect();

        Ok(oldest)
    }

    fn get_oldest_files_by_max_age(
        &self,
        glob_pattern: &str,
    ) -> Result<Vec<PathBuf>, Box<dyn Error>> {
        let age_duration = match self.parse_age_to_duration() {
            Some(dur) => dur,
            None => return Ok(vec![]),
        };

        let cutoff = Utc::now().timestamp() - age_duration.num_seconds();

        let re = Regex::new(r"(\d{4}-\d{2}-\d{2}_\d{2}-\d{2}-\d{2})")?;

        let old_files: Vec<PathBuf> = glob(glob_pattern)?
            .filter_map(Result::ok)
            .filter_map(|path| {
                self.extract_datetime_from_filename(&path, &re)
                    .filter(|&ts| ts < cutoff)
                    .map(|_| path)
            })
            .collect();

        Ok(old_files)
    }

    fn extract_numeric_timestamp_from_filename(path: &PathBuf, regex: &Regex) -> Option<u64> {
        if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
            if let Some(caps) = regex.captures(filename) {
                let raw = caps.get(1)?.as_str();
                let numeric: String = raw.chars().filter(|c| c.is_numeric()).collect();
                return numeric.parse::<u64>().ok();
            }
        }
        None
    }

    fn extract_datetime_from_filename(&self, path: &PathBuf, re: &Regex) -> Option<i64> {
        let filename = path.file_name()?.to_str()?;
        let caps = re.captures(filename)?;
        let timestamp_str = caps.get(1)?.as_str();

        let datetime = NaiveDateTime::parse_from_str(timestamp_str, "%Y-%m-%d_%H-%M-%S").ok()?;
        Some(datetime.and_utc().timestamp())
    }

    fn parse_age_to_duration(&self) -> Option<Duration> {
        let age = self.config.delete_backup_after.clone().unwrap_or_default();
        let re = Regex::new(r"^(\d+)([dwmy])$").ok()?;
        let caps = re.captures(&*age)?;

        let value: i64 = caps.get(1)?.as_str().parse().ok()?;
        let unit = caps.get(2)?.as_str();

        match unit {
            "d" => Some(Duration::days(value)),
            "w" => Some(Duration::weeks(value)),
            "m" => Some(Duration::days(30 * value)),
            "y" => Some(Duration::days(365 * value)),
            _ => None,
        }
    }
}
