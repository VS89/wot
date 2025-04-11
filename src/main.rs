pub mod external_api;
pub mod utils;
pub mod constants;
pub mod command_logic;
pub mod config;
pub mod cli_app;
pub mod create_template;

use clap::Parser;
use directories::UserDirs;
use external_api::ApiError;
use external_api::testops_api::TestopsApi;
use std::fs::File;
use std::path::Path;
use cli_app::{Cli, Commands};
use command_logic::report::send_report;
use command_logic::testcase::import_testcase_by_id;
use config::Config;
use constants::CONFIG_DIR;

#[tokio::main]
async fn main() -> Result<(), ApiError> {
    let mut config_path = Path::new("config.json");
    if cfg!(debug_assertions) {
        config_path = Path::new("test_config.json");
    }
    if let Some(user_dirs) = UserDirs::new() {
        let path = user_dirs.home_dir().join(CONFIG_DIR).join(config_path);
        if path.exists() {
            let config = Config::get_config(path)?;
            let testops_api = TestopsApi::new(&config.testops_api_token, &config.testops_base_url);
            let cli = Cli::parse();
            let stdin = std::io::stdin();
            let stdout = std::io::stdout();

            match &cli.command {
                Commands::Report(value) => {
                    match send_report(&value.directory_path, value.project_id, &testops_api, stdin.lock(), stdout).await {
                        Ok(value) => println!("{}", value),
                        Err(e) => eprintln!("Failed to send report: {}", e),
                    };
                }
                Commands::Testcase(value) => {
                    let _ = import_testcase_by_id(value.import_testcase_id, &testops_api)
                        .await
                        .map_err(|e| eprintln!("Failed to import testcase by id: {}", e));
                }
            }
        } else {
            let app = Config::new()?;
            if let Some(parent_dir) = path.parent() {
                std::fs::create_dir_all(parent_dir)?;
            }
            let file = File::create(path)?;
            serde_json::to_writer_pretty(file, &app).unwrap_or_else(
                |_| panic!("{}", ApiError::CantCreateConfig.to_string()))
        }
    }
    Ok(())
}
