use clap::Parser;
use directories::UserDirs;
use wot::external_api::base_api_client::ApiError;
use wot::external_api::testops_api::testops_api::TestopsApi;
use std::fs::File;
use std::path::Path;
use wot::cli_app::{Cli, Commands};
use wot::command_logic::report::send_report;
use wot::command_logic::testcase::import_testcase_by_id;
use wot::config::Config;
use wot::constants::CONFIG_DIR;
use wot::errors::CANT_CREATE_CONFIG;

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

            match &cli.command {
                Commands::Report(value) => {
                    send_report(&value.directory_path, value.project_id, &testops_api).await?
                }
                Commands::Testcase(value) => {
                    import_testcase_by_id(value.import_testcase_id, &testops_api).await?
                }
            }
        } else {
            let app = Config::new()?;
            if let Some(parent_dir) = path.parent() {
                std::fs::create_dir_all(parent_dir)?;
            }
            let file = File::create(path)?;
            serde_json::to_writer_pretty(file, &app).expect(CANT_CREATE_CONFIG)
        }
    }
    Ok(())
}
