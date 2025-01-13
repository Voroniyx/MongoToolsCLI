use crate::utils::{append_timestamp, create_tar_gz, delete_dir, ensure_dir_exists, generate_file};
use colored::Colorize;
use core::result::Result::Err;
use futures::stream::StreamExt;
use mongodb::bson::{doc, Document};
use mongodb::error::Error;
use mongodb::Client;
use regex::Regex;
use tokio::io::AsyncWriteExt;

pub async fn create_backup(connection_string: &str) -> Result<(), Error> {
    let client_result = Client::with_uri_str(connection_string).await;
    let db_name = extract_db_from_connection_string(connection_string);

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
                ensure_dir_exists("out").join(format!("{}.tar.gz", append_timestamp(&db_name)));

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
                println!("{}", "Backup wurde unter '/out/' erstellt".green());
            }
        }
        Err(e) => {
            println!(
                "{}, {:?}",
                "Während der Backup erstellung ist ein Fehler aufgetreten ".red(),
                e
            )
        }
    }
}

pub fn extract_db_from_connection_string(connection_string: &str) -> String {
    let re = Regex::new(
        r"mongodb://([^:]+):([^@]+)@([^:/]+)(?::(\d+))?/([^?]+)\?retryWrites=true&w=majority",
    )
    .unwrap();

    let mut db_name = String::new();

    if let Some(captures) = re.captures(connection_string) {
        // let db_user = captures.get(1).map_or("", |m| m.as_str());
        // let db_password = captures.get(2).map_or("", |m| m.as_str());
        // let ip_or_host = captures.get(3).map_or("", |m| m.as_str());
        // let port = captures.get(4).map_or("", |m| m.as_str());
        db_name = captures
            .get(5)
            .map_or("".parse().unwrap(), |m| m.as_str().parse().unwrap());
    }

    db_name
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


