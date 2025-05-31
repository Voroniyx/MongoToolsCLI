use crate::cleanup_manager::CleanUpManager;
use crate::config::Config;
use crate::log::Log;
use crate::utils::Utils;
use chrono::Utc;
use colored::Colorize;
use core::result::Result::Err;
use cron::Schedule;
use futures::stream::StreamExt;
use mongodb::bson::{doc, Document};
use mongodb::error::Error as MongoError;
use mongodb::Client;
use regex::Regex;
use std::error::Error;
use std::str::FromStr;
use tokio::io::AsyncWriteExt;
use url::Url;

pub struct BackupManager {
    pub config: Config,
}

impl BackupManager {
    // Creates a single backup from the provided config
    // and returns the file name when the operation was successful
    pub async fn create(&self) -> Result<String, MongoError> {
        self.create_backup().await
    }

    pub async fn cron(&self) {
        Log::info("Started cron job");

        let cron_expression = self
            .config
            .cron_job_expression
            .clone()
            .expect("No cron job expression");

        let schedule = Schedule::from_str(cron_expression.as_str()).expect("Invalid cron job time");
        loop {
            if let Some(job_time) = schedule.upcoming(Utc).take(1).next() {
                let until_next = job_time - Utc::now();
                tokio::time::sleep(until_next.to_std().unwrap()).await;
                println!(
                    "{} {}",
                    Utils::get_readable_timestamp(),
                    "Starting Backup...".blue()
                );

                let backup = BackupManager {
                    config: self.config.clone(),
                };
                let backup_result = backup.create_backup().await;
                self.handle_backup_result(backup_result, true).await;
                println!(
                    "{} {}",
                    Utils::get_readable_timestamp(),
                    "Backup finished!".green()
                );
            }
        }
    }

    // Creates a single backup from the provided config
    // and returns the file name when the operation was successful
    async fn create_backup(&self) -> Result<String, MongoError> {
        let connection_string = self
            .config
            .connection_string
            .clone()
            .expect("Connection string not found");

        let output_path = self
            .config
            .tar_gz_path
            .clone()
            .expect("Targz path not found");

        let client_result = Client::with_uri_str(&connection_string).await;
        let db_name = BackupManager::get_mongodb_database_name(&connection_string);

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

                    if let Err(e) = BackupManager::process_cursor(cursor, file).await {
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

                let _ = Utils::create_tar_gz(
                    Utils::ensure_dir_exists("temp").as_path(),
                    archive_file_name.clone(),
                );

                Utils::delete_dir(&temp_dir).expect("Could not delete temp dir");

                Ok(archive_file_name
                    .clone()
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_owned())
            }
            Err(e) => Err(e),
        }
    }

    pub async fn handle_backup_result(
        &self,
        result: Result<String, MongoError>,
        suppress_ok_msg: bool,
    ) {
        let out_dir = self
            .config
            .tar_gz_path
            .clone()
            .unwrap_or("/".parse().unwrap());
        match result {
            Ok(_file_name) => {
                let clean_up_manager = CleanUpManager {
                    config: self.config.clone(),
                };
                clean_up_manager
                    .by_max_concurrent_backups(self.config.tar_gz_path.clone())
                    .await;

                clean_up_manager
                    .by_max_age(self.config.tar_gz_path.clone())
                    .await;

                if !suppress_ok_msg {
                    println!("{}", format!("Backup created under '{}'", out_dir).green());
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
    ) -> Result<(), Box<dyn Error>> {
        while let Some(result) = cursor.next().await {
            let document = result?;
            let json = serde_json::to_string(&document)?;
            file.write_all(json.as_bytes()).await?;
            file.write_all(b"\n").await?;
        }
        Ok(())
    }
}
