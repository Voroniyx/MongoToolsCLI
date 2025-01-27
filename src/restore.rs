use crate::backup::extract_db_from_connection_string;
use crate::config::Config;
use colored::Colorize;
use flate2::read::GzDecoder;
use mongodb::bson::Document;
use mongodb::{bson, Client};
use serde_json::Value;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use tar::Archive;

struct CollectionData {
    pub name: String,
    pub data: String,
}

pub async fn restore_from_targz(config: Config) {
    let archive_path = config.targz_path.expect("Tar GZ Path not found");
    let file_contents = get_content_from_targz_inner_files(&archive_path).await;

    let conn_string = config.connection_string.expect("Connection String not found");
    let client_result = Client::with_uri_str(&conn_string).await;

    if !client_result.is_ok() {
        println!("Could not create DB Client: {}", client_result.unwrap_err());
        return;
    }

    let client = client_result.unwrap();

    match file_contents {
        Ok(contents) => {
            for collection_data in contents {
                handle_single_collection(
                    collection_data,
                    &client,
                    extract_db_from_connection_string(&conn_string).as_str(),
                )
                .await;
            }
        }
        Err(e) => {
            eprintln!("{}", e);
        }
    }
}

async fn get_content_from_targz_inner_files(
    targz_path: &str,
) -> Result<Vec<CollectionData>, String> {
    if !Path::new(targz_path).exists() {
        return Err(String::from("Targz File not found"));
    }

    let file = File::open(targz_path).unwrap();

    let decoder = GzDecoder::new(file);
    let mut archive = Archive::new(decoder);

    let mut file_contents: Vec<CollectionData> = Vec::new();

    for entry in archive.entries().unwrap() {
        let mut entry = entry.unwrap();
        if entry.header().entry_type().is_file() {
            let mut contents = String::new();
            entry.read_to_string(&mut contents).unwrap();
            let data = CollectionData {
                data: contents,
                name: String::from(entry.path().unwrap().file_stem().unwrap().to_str().unwrap()),
            };
            file_contents.push(data)
        }
    }

    Ok(file_contents)
}

async fn handle_single_collection(
    collection_data: CollectionData,
    mongo_client: &Client,
    database_name: &str,
) {
    let database = mongo_client.database(database_name);
    let collection = database.collection::<Document>(collection_data.name.as_str());

    let stringified_documents: Vec<&str> = collection_data
        .data
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect();

    let mut bson_documents = Vec::new();
    for doc in &stringified_documents {
        let json: Value = serde_json::from_str(doc).unwrap();
        let bson_doc: Document = bson::to_document(&json).unwrap();
        bson_documents.push(bson_doc);
    }

    let insert_result = collection.insert_many(&bson_documents).await;

    if !insert_result.is_ok() {
        eprintln!(
            "Could not insert documents in collection: '{}' due to {}",
            &collection_data.name,
            insert_result.unwrap_err()
        );
    } else {
        let insert = insert_result.unwrap();
        println!(
            "{}",
            format!(
                "Successfully inserted {} items into '{}' collection",
                &insert.inserted_ids.len(),
                collection_data.name
            )
            .green()
        );
    }
}
