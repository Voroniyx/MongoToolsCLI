use crate::backup::Backup;
use crate::config::{Config, ConfigLoadError};
use crate::log::Log;
use crate::restore::restore_from_targz;
use crate::utils::Utils;
use chrono::Utc;
use colored::Colorize;
use cron::Schedule;
use std::cmp::PartialEq;
use std::str::FromStr;

mod backup;
mod config;
mod log;
mod restore;
mod utils;

#[derive(Debug, PartialEq)]
enum CliMode {
    Backup,
    Restore,
}

#[tokio::main]
async fn main() {
    Log::info("Initializing...");
    match Config::load().await {
        Ok(configs) => {
            Log::info(&format!("Loaded {} config entries.", configs.len()));
            let single_config = configs.len() == 1;
            if single_config {
                let first_config = configs.first().unwrap();

                let cli_force = first_config.force_cli.unwrap_or(false);
                if cli_force {
                    //cli with config
                    manual(configs.first().cloned()).await;
                } else {
                    //Cron Mode
                    cron(configs).await;
                }
            } else {
                cron(configs).await;
            }
        }
        Err(ConfigLoadError::NotFound) => {
            //CLI Mode
            manual(None).await;
        }
        Err(ConfigLoadError::ParseError(err)) => {
            eprintln!("{}", err);
        }
    }
}

async fn manual(config: Option<Config>) {
    Log::info("MongoDB CLI Tools");
    Log::success("Following features are available");
    Log::success("- backup");
    Log::success("- restore");

    let input = Utils::ask(&*format!("{}", "Which feature should executed".magenta()));

    if input == "backup" {
        if config.is_none() {
            let ask_for_config_result = ask_user_for_config(CliMode::Backup);

            match ask_for_config_result {
                Ok(user_provided_config) => {
                    let backup_result = Backup::create_backup(
                        user_provided_config.connection_string,
                        user_provided_config.targz_path,
                        user_provided_config.name,
                    )
                    .await;
                    Backup::handle_backup_result(backup_result, false);
                }
                Err(err) => {
                    println!("{}", err.red());
                }
            }
        } else {
            let cnf = config.unwrap();
            let backup_result =
                Backup::create_backup(cnf.connection_string, cnf.targz_path, cnf.name).await;
            Backup::handle_backup_result(backup_result, false);
        }
    } else if input == "restore" {
        if config.is_none() {
            let ask_for_config_result = ask_user_for_config(CliMode::Restore);

            match ask_for_config_result {
                Ok(user_provided_config) => {
                    restore_from_targz(user_provided_config).await;
                }
                Err(err) => {
                    Log::error(&*err);
                }
            }
        } else {
            let unwrapped_config = config.unwrap();
            restore_from_targz(unwrapped_config).await;
        }
    } else {
        Log::error("No matching option provided");
    }
}

async fn cron(configs: Vec<Config>) {
    Log::info("MongoDB CLI Tools Cron Job started");

    let handles: Vec<_> = configs
        .into_iter()
        .map(|config| {
            tokio::spawn(async move {
                let con_str = config.connection_string.expect("No connection string");
                let cron_expression = config.cron_job_expression.expect("No cron job expression");
                let output_path = config.targz_path.expect("No output path found");

                let connection_string = Utils::trim_double_quotes_chars(con_str);
                let schedule = Schedule::from_str(&cron_expression).expect("Invalid cron job time");

                loop {
                    if let Some(job_time) = schedule.upcoming(Utc).take(1).next() {
                        let until_next = job_time - Utc::now();
                        tokio::time::sleep(until_next.to_std().unwrap()).await;
                        Log::info(&format!("{} Starting Backup...", config.name.clone()));

                        let backup_result = Backup::create_backup(
                            Option::from(connection_string.clone()),
                            Option::from(output_path.clone()),
                            config.name.clone(),
                        )
                        .await;
                        Backup::handle_backup_result(backup_result, true);
                        Log::success(&format!("{} Backup finished!", config.name.clone()));
                    }
                }
            })
        })
        .collect();

    for handle in handles {
        let _ = handle.await;
    }
}

fn ask_user_for_config(mode: CliMode) -> Result<Config, String> {
    let mut config = Config {
        connection_string: None,
        targz_path: None,
        force_cli: None,
        cron_job_expression: None,
        name: "".to_string(),
    };

    config.name = Utils::ask(&*format!(
        "{}",
        "Please provide a name for the config".magenta()
    ));

    if Some(config.name.clone()).is_none() {
        return Err(String::from("Name is empty"));
    }

    config.connection_string = Option::from(Utils::ask(&*format!(
        "{}",
        "Please provide a mongo db connection string".magenta()
    )));

    if Some(config.connection_string.as_ref().unwrap()).is_none() {
        return Err(String::from("Connection string is empty"));
    }

    if mode == CliMode::Restore {
        config.targz_path = Option::from(Utils::ask(&*format!(
            "{}",
            "Please provide a tar gz path".magenta()
        )));
        if Some(config.targz_path.as_ref().unwrap()).is_none() {
            return Err(String::from("Tar GZ Path is empty but required."));
        }
    }

    Ok(config)
}
