use chrono::Local;
use flate2::write::GzEncoder;
use flate2::Compression;
use mongodb::error::Error;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::{env, fs, io};
use tar::Builder;

pub struct Utils {}

impl Utils {
    pub fn ask(question: &str) -> String {
        println!("{}", question);
        let mut input = String::new();

        io::stdin()
            .read_line(&mut input)
            .expect("Failed to parse field");

        let formated_input = input.trim().replace("\n", "").replace("\r", "");

        formated_input
    }

    pub fn get_readable_timestamp() -> String {
        let timestamp = Local::now().format("%Y-%m-%d %H-%M-%S").to_string();
        format!("[{}]", timestamp)
    }

    pub fn get_project_root_path(relative_path: &str) -> PathBuf {
        let current_dir = env::current_dir().expect("Failed to get current directory");
        current_dir.join(relative_path)
    }

    pub fn ensure_dir_exists(path: &str) -> PathBuf {
        let full_path = Utils::get_project_root_path(path);

        if !&full_path.exists() {
            fs::create_dir_all(&full_path)
                .expect(&*("Could not create dir for path ".to_owned() + path));
        }

        full_path
    }

    pub async fn generate_file(file_name: String, path: &PathBuf) -> io::Result<tokio::fs::File> {
        let f_n = &path.join(&file_name);
        tokio::fs::File::create(&f_n).await
    }

    pub fn delete_dir(path: &PathBuf) -> io::Result<()> {
        fs::remove_dir_all(path)
    }

    pub fn create_tar_gz(src_dir: &Path, output_file: PathBuf) -> Result<(), Error> {
        let tar_gz_file = File::create(output_file)?;
        let encoder = GzEncoder::new(tar_gz_file, Compression::best());
        let mut tar = Builder::new(encoder);

        tar.append_dir_all(".", src_dir)?;
        Ok(())
    }

    pub fn append_timestamp(input: &str) -> String {
        let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();
        format!("{}_{}", input, timestamp)
    }

    pub fn trim_double_quotes_chars(input: String) -> String {
        let mut chars = input.chars();

        if chars.next() == Some('"') && chars.next_back() == Some('"') {
            chars.collect()
        } else {
            input.to_string()
        }
    }
}
