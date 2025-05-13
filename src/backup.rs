use crate::log::Log;
use crate::utils::Utils;
use colored::Colorize;
use core::result::Result::Err;
use cron::TimeUnitSpec;
use futures::stream::StreamExt;
use glob::{glob, Paths};
use mongodb::bson::{doc, Document};
use mongodb::error::Error as MongoError;
use mongodb::Client;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt::format;
use std::path::PathBuf;
use tokio::io::AsyncWriteExt;
use url::Url;

pub struct Backup {}

impl Backup {
    pub async fn create_backup(
        connection_string_option: Option<String>,
        targz_path: Option<String>,
        max_concurrent_backups: Option<usize>,
    ) -> Result<(), MongoError> {
        let connection_string = connection_string_option.expect("Connection string not found");
        let output_path = targz_path.expect("Targz path not found");
        let client_result = Client::with_uri_str(&connection_string).await;
        let db_name = Backup::get_mongodb_database_name(&connection_string);

        match client_result {
            Ok(client) => {
                let db = client.database(&db_name);
                let temp_dir = Utils::ensure_dir_exists("temp");

                let collection_names = db.list_collection_names().await?;
                for collection_name in collection_names {
                    let collection = db.collection::<Document>(&collection_name);

                    let file_name = format!("{}.json", collection_name);
                    let file = Utils::generate_file(file_name, &temp_dir).await?;

                    let filter = doc! {};
                    let cursor = collection.find(filter).await?;

                    if let Err(e) = Backup::process_cursor(cursor, file).await {
                        eprintln!(
                            "{} '{}': {}",
                            "Error processing cursor for collection ".red(),
                            collection_name,
                            e
                        );
                    }
                }

                let archive_file_name = Utils::ensure_dir_exists(&output_path)
                    .join(format!("{}.tar.gz", Utils::append_timestamp(&db_name)));

                let archive = Utils::create_tar_gz(
                    Utils::ensure_dir_exists("temp").as_path(),
                    archive_file_name,
                );

                Utils::delete_dir(&temp_dir).expect("Could not delete temp dir");

                archive
            }
            Err(e) => Err(e),
        }
    }

    pub fn handle_backup_result(result: Result<(), MongoError>, suppress_ok_msg: bool) {
        match result {
            Ok(..) => {
                if !suppress_ok_msg {
                    println!("{}", "Backup created under '/out/'".green());
                }
            }
            Err(e) => {
                println!(
                    "{}, {:?}",
                    "An error occurred during backup creation".red(),
                    e
                )
            }
        }
    }

    pub fn extract_db_from_connection_string(connection_string: &str) -> String {
        let re = Regex::new(r"mongodb\+srv?://(?:([^:]+):([^@]+)@)?([\w.-]+)/([^?]+)(?:\?(.*))?")
            .unwrap();

        if let Some(captures) = re.captures(connection_string) {
            return captures
                .get(4)
                .map_or_else(|| "".to_string(), |m| m.as_str().to_string());
        }

        "".to_string()
    }

    pub fn get_mongodb_database_name(connection_string: &str) -> String {
        if let Ok(url) = Url::parse(connection_string) {
            if let Some(db_name) = url.path_segments().and_then(|segments| segments.last()) {
                if !db_name.is_empty() {
                    return db_name.to_string();
                }
            }
        }

        "".to_string()
    }

    async fn process_cursor(
        mut cursor: mongodb::Cursor<Document>,
        mut file: tokio::fs::File,
    ) -> Result<(), Box<dyn std::error::Error>> {
        while let Some(result) = cursor.next().await {
            let document = result?;
            let json = serde_json::to_string(&document)?;
            file.write_all(json.as_bytes()).await?;
            file.write_all(b"\n").await?;
        }
        Ok(())
    }

    pub async fn check_max_concurrent_backups(path: &str, max_concurrent_backups: Option<usize>) {
        let files_result = Self::get_oldest_files(
            format!(
                "{}/*.tar.gz",
                Utils::get_project_root_path(path)
                    .display()
                    .to_string()
                    .as_str()
            )
            .as_str(),
            max_concurrent_backups.unwrap_or(0),
        );

        match files_result {
            Ok(files) => {
                for file in files {
                    let delete_result = Utils::delete_file(&file).await;
                    match delete_result {
                        Ok(_) => {}
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

    fn extract_timestamp(path: &PathBuf, regex: &Regex) -> Option<u64> {
        if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
            if let Some(caps) = regex.captures(filename) {
                let raw = caps.get(1)?.as_str();
                let numeric: String = raw.chars().filter(|c| c.is_numeric()).collect();
                return numeric.parse::<u64>().ok();
            }
        }
        None
    }

    fn get_oldest_files(
        glob_pattern: &str,
        max_concurrent_backups: usize,
    ) -> Result<Vec<PathBuf>, Box<dyn Error>> {
        let re = Regex::new(r"(\d{4}-\d{2}-\d{2}_\d{2}-\d{2}-\d{2})")?;

        let mut files_with_timestamps: Vec<(PathBuf, u64)> = glob(glob_pattern)?
            .filter_map(Result::ok)
            .filter_map(|path| Self::extract_timestamp(&path, &re).map(|ts| (path, ts)))
            .collect();

        files_with_timestamps.sort_by_key(|(_, ts)| *ts);

        let oldest = files_with_timestamps
            .into_iter()
            .take(max_concurrent_backups)
            .map(|(path, _)| path)
            .collect();

        Ok(oldest)
    }
}
