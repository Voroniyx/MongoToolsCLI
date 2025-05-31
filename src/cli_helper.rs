use crate::utils::Utils;
use colored::Colorize;
use crate::config::Config;

pub struct CliHelper {}

#[derive(Debug, PartialEq)]
pub enum CliMode {
    Backup,
    Restore,
}

impl CliHelper {
    pub fn ask_mode() -> Result<CliMode, ()> {
        let answer = Utils::ask(&*format!("{}", "Which feature should i execute. Options: 'backup' | 'restore'".magenta()));
        if answer == "backup" {
            Ok(CliMode::Backup)
        } else if answer == "restore" {
            Ok(CliMode::Restore)
        } else {
            Err(())
        }
    }

    pub fn ask_config() -> Result<Config, String> {
        let mut config = Config {
            connection_string: None,
            tar_gz_path: None,
            cron_job_expression: None,
            max_concurrent_backups: None,
            delete_backup_after: None,
        };

        config.connection_string = Option::from(Utils::ask(&*format!(
            "{}",
            "Please provide a mongo db connection string".magenta()
        )));

        if Some(config.connection_string.as_ref().unwrap()).is_none() {
            return Err(String::from("Connection string is empty"));
        }

        config.tar_gz_path = Option::from(Utils::ask(&*format!(
            "{}",
            "Please provide a tar gz path".magenta()
        )));

        if Some(config.tar_gz_path.as_ref().unwrap()).is_none() {
            return Err(String::from("Tar GZ Path is empty but required."))
        }

        Ok(config)
    }
}
