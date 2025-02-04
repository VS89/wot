use clap::Parser;
use directories::UserDirs;
use std::fs::File;
use std::path::Path;
use wot::cli_app::{Cli, Commands};
use wot::config::Config;
use wot::constants::CONFIG_DIR;
use wot::errors::CANT_CREATE_CONFIG;
use wot::send_report;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut config_path = Path::new("config.json");
    if cfg!(debug_assertions) {
        config_path = Path::new("test_config.json");
    }
    if let Some(user_dirs) = UserDirs::new() {
        let path = user_dirs.home_dir().join(CONFIG_DIR).join(config_path);
        if path.exists() {
            let config = Config::get_config(path)?;
            let cli = Cli::parse();

            match &cli.command {
                Commands::Report(value) => {
                    send_report(&value.directory_path, value.project_id, &config).await?
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
