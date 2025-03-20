use std::cmp::PartialEq;
use crate::backup::{create_backup, handle_backup_result};
use crate::config::{Config, ConfigLoadError};
use crate::restore::restore_from_targz;
use crate::utils::{ask, get_readable_timestamp, trim_double_quotes_chars};
use chrono::Utc;
use colored::Colorize;
use cron::Schedule;
use std::str::FromStr;

mod backup;
mod config;
mod restore;
mod utils;

#[derive(Debug, PartialEq)]
enum CliMode {
    Backup,
    Restore,
}


#[tokio::main]
async fn main() {
    match Config::load().await {
        Ok(config) => {
            let cli_force = config.force_cli.unwrap_or(false);
            if cli_force {
                //cli with config
                manual(Some(config)).await;
            } else {
                //Cron Mode
                cron(config).await;
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
    println!("{}", "MongoDB CLI Tools".green().bold());
    println!("{}", "Following features are available".green());
    println!("{}", "- backup".cyan());
    println!("{}", "- restore".cyan());

    let input = ask(&*format!(
        "{}",
        "Which feature should executed".magenta()
    ));

    if input == "backup" {
        if config.is_none() {
            let ask_for_config_result = ask_user_for_config(CliMode::Backup);

            match ask_for_config_result {
                Ok(user_provided_config) => {
                    let backup_result = create_backup(user_provided_config.connection_string, user_provided_config.targz_path).await;
                    handle_backup_result(backup_result, false);
                }
                Err(err) => {
                    println!("{}", err.red());
                }
            }

        } else {
            let cnf = config.unwrap();
            let backup_result = create_backup(cnf.connection_string, cnf.targz_path).await;
            handle_backup_result(backup_result, false);
        }


    } else if input == "restore" {
        if config.is_none() {
            let ask_for_config_result = ask_user_for_config(CliMode::Restore);

            match ask_for_config_result {
                Ok(user_provided_config) => {
                    restore_from_targz(user_provided_config).await;
                }
                Err(err) => {
                    println!("{}", err.red());
                }
            }

        } else {
            let unwrapped_config = config.unwrap();
            restore_from_targz(unwrapped_config).await;
        }


    } else {
        println!("{}", "No matching option provided".red())
    }
}

async fn cron(config: Config) {
    println!(
        "{} {}",
        get_readable_timestamp(),
        "MongoDB CLI Tools Cron Job started".green().bold()
    );

    let con_str = config.connection_string.expect("No connection string");
    let cron_expression = config.cron_job_expression.expect("No cron job expression");
    let output_path = config.targz_path.expect("No output path found");

    let connection_string = trim_double_quotes_chars(con_str);
    let schedule = Schedule::from_str(&cron_expression).expect("Invalid cron job time");
    loop {
        if let Some(job_time) = schedule.upcoming(Utc).take(1).next() {
            let until_next = job_time - Utc::now();
            tokio::time::sleep(until_next.to_std().unwrap()).await;
            println!(
                "{} {}",
                get_readable_timestamp(),
                "Starting Backup...".blue()
            );
            let backup_result = create_backup(Option::from(connection_string.clone()), Option::from(output_path.clone())).await;
            handle_backup_result(backup_result, true);
            println!(
                "{} {}",
                get_readable_timestamp(),
                "Backup finished!".green()
            );
        }
    }
}



fn ask_user_for_config(mode: CliMode) -> Result<Config, String> {
    let mut config = Config {
        connection_string: None,
        targz_path: None,
        force_cli:None,
        cron_job_expression: None,
    };

    config.connection_string = Option::from(ask(&*format!(
        "{}",
        "Please provide a mongo db connection string".magenta()
    )));

    if Some(config.connection_string.as_ref().unwrap()).is_none() {
        return Err(String::from("Connection string is empty"));
    }

    if mode == CliMode::Restore {
        config.targz_path = Option::from(ask(&*format!(
            "{}",
            "Please provide a tar gz path".magenta()
        )));
        if Some(config.targz_path.as_ref().unwrap()).is_none() {
            return Err(String::from("Tar GZ Path is empty but required."))
        }
    }

    Ok(config)
}