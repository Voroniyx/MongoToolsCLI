use serde::Deserialize;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub cron_job_expression: Option<String>,
    pub connection_string: Option<String>,
    pub tar_gz_path: Option<String>,
    pub max_concurrent_backups: Option<usize>,
    pub delete_backup_after: Option<String>, //default 90 days
}

#[derive(Debug)]
pub enum ConfigLoadError {
    NotFound,
    ParseError(String),
}

impl Config {
    pub async fn load() -> Result<Config, ConfigLoadError> {
        let config_path = Path::new("./config.json");

        let config_file = File::open(&config_path).map_err(|_| ConfigLoadError::NotFound)?;

        let config_file_reader = BufReader::new(config_file);

        let config = serde_json::from_reader(config_file_reader)
            .map_err(|e| ConfigLoadError::ParseError(format!("Failed to parse JSON: {}", e)))?;

        Ok(config)
    }

    pub fn clone(&self) -> Config {
        Config {
            connection_string: self.connection_string.clone(),
            max_concurrent_backups: self.max_concurrent_backups.clone(),
            tar_gz_path: self.tar_gz_path.clone(),
            cron_job_expression: self.cron_job_expression.clone(),
            delete_backup_after: self.delete_backup_after.clone(),
        }
    }
}
