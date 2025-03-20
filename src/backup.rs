use crate::utils::{append_timestamp, create_tar_gz, delete_dir, ensure_dir_exists, generate_file};
use colored::Colorize;
use core::result::Result::Err;
use futures::stream::StreamExt;
use mongodb::bson::{doc, Document};
use mongodb::error::Error;
use mongodb::Client;
use regex::Regex;
use tokio::io::AsyncWriteExt;
use url::Url;

pub async fn create_backup(connection_string_option: Option<String>, targz_path:Option<String>) -> Result<(), Error> {
    let connection_string = connection_string_option.expect("Connection string not found");
    let output_path = targz_path.expect("Targz path not found");
    let client_result = Client::with_uri_str(&connection_string).await;
    let db_name = get_mongodb_database_name(&connection_string);

    match client_result {
        Ok(client) => {
            let db = client.database(&db_name);
            let temp_dir = ensure_dir_exists("temp");

            let collection_names = db.list_collection_names().await?;
            for collection_name in collection_names {
                let collection = db.collection::<Document>(&collection_name);

                let file_name = format!("{}.json", collection_name);
                let file = generate_file(file_name, &temp_dir).await?;

                let filter = doc! {};
                let cursor = collection.find(filter).await?;

                if let Err(e) = process_cursor(cursor, file).await {
                    eprintln!(
                        "{} '{}': {}",
                        "Error processing cursor for collection ".red(),
                        collection_name,
                        e
                    );
                }
            }

            let archive_file_name =
                ensure_dir_exists(&output_path).join(format!("{}.tar.gz", append_timestamp(&db_name)));

            let archive = create_tar_gz(ensure_dir_exists("temp").as_path(), archive_file_name);

            delete_dir(&temp_dir).expect("Could not delete temp dir");

            archive
        }
        Err(e) => Err(e),
    }
}

pub fn handle_backup_result(result: Result<(), Error>, suppress_ok_msg: bool) {
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
    let re =
        Regex::new(r"mongodb\+srv?://(?:([^:]+):([^@]+)@)?([\w.-]+)/([^?]+)(?:\?(.*))?").unwrap();

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
