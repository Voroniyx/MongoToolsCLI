use crate::backup_manager::BackupManager;
use crate::cli_helper::{CliHelper, CliMode};
use crate::config::Config;
use crate::log::Log;
use crate::restore_manager::RestoreManager;
use crate::utils::Utils;

mod backup_manager;
mod cli_helper;
mod config;
mod log;
mod restore_manager;
mod utils;
mod cleanup_manager;

#[tokio::main]
async fn main() {
    // 1) read args
    //    2) has cron flag
    //       - load config
    //       - run cron job
    //    3) else
    //       - run cli
    //         - switch answer
    //            - backup
    //            - restore

    let load_result = Config::load().await;
    if Utils::has_cron_arg() {
        // run cron job
        match load_result {
            Ok(config) => {
                let backup_manager = BackupManager { config };
                backup_manager.cron().await;
            }
            Err(err) => {
                panic!("{:?}", err);
            }
        }
    } else {
        match load_result {
            Ok(config) => {
                // ask what to do with config from file
                let mode_answer = CliHelper::ask_mode();
                handle_mode(mode_answer, config).await;
            }
            Err(_err) => {
                // ask for config via cli
                let config_answer = CliHelper::ask_config();
                match config_answer {
                    Ok(config) => {
                        // ask what to do with config from cli
                        let mode_answer = CliHelper::ask_mode();
                        handle_mode(mode_answer, config).await;
                    }
                    Err(err) => {
                        panic!("{:?}", err);
                    }
                }
            }
        }
    }
}

async fn handle_mode(mode_answer: Result<CliMode, ()>, config: Config) {
    match mode_answer {
        Ok(mode) => {
            if mode == CliMode::Backup {
                let backup_manager = BackupManager { config };
                let backup_result = backup_manager.create().await;
                backup_manager.handle_backup_result(backup_result, false).await;
            } else if mode == CliMode::Restore {
                let restore_manager = RestoreManager { config };
                restore_manager.restore().await;
            } else {
                Log::error("No valid mode selected")
            }
        }
        Err(_err) => Log::error("No valid mode selected"),
    }
}
