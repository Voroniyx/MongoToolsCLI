use crate::backup::{create_backup, handle_backup_result};
use crate::utils::{ask, get_readable_timestamp};
use chrono::Utc;
use colored::Colorize;
use cron::Schedule;
use std::str::FromStr;

mod backup;
mod utils;

#[tokio::main]
async fn main() {
    let run_as_cron_job_env = std::env::var("MONGOTOOLS_CRON_JOB");

    match run_as_cron_job_env {
        Ok(cron_expression) => {
            cron(cron_expression).await;
        }
        _ => {
            manual().await;
        }
    }
}

async fn manual() {
    println!("{}", "MongoDB CLI Tools".green().bold());
    println!("{}", "Folgende Funktionen sind möglich:".green());
    println!("{}", "- backup".cyan());
    println!("{}", "- restore (WIP)".cyan());

    let input = ask(&*format!(
        "{}",
        "Welche Funktion soll ausgeführt werden?".magenta()
    ));

    if input == "backup" {
        let connection_string = ask(&*format!(
            "{}",
            "Bitte gib einen Connection String an:".magenta()
        ));
        if connection_string.is_empty() {
            println!("{}", "Connection string ist leer".red());
            return;
        }

        let backup_result = create_backup(&*connection_string).await;
        handle_backup_result(backup_result, false);
    } else if input == "restore" {
        println!("{}", "Restore ist aktuell ein Work in progress".yellow());
    } else {
        println!("{}", "Keine passende Option angegeben".red())
    }
}

async fn cron(cron_expression: String) {
    println!(
        "{} {}",
        get_readable_timestamp(),
        "MongoDB CLI Tools Cron Job started".green().bold()
    );

    let connection_string_env = std::env::var("MONGOTOOLS_CONNECTION_STRING");

    match connection_string_env {
        Ok(connection_string) => {
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
                    let backup_result = create_backup(&connection_string).await;
                    handle_backup_result(backup_result, true);
                    println!(
                        "{} {}",
                        get_readable_timestamp(),
                        "Backup finished!".green()
                    );
                }
            }
        }
        Err(_) => {
            panic!("MONGOTOOLS_CONNECTION_STRING is not set");
        }
    }
}
