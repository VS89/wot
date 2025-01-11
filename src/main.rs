use clap::Parser;
use directories::UserDirs;
use std::fs::File;
use tokio;
use wot::cli_app::{Cli, Commands};
use wot::config::Config;
use wot::send_report;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    if let Some(user_dirs) = UserDirs::new() {
        let path = user_dirs.home_dir().join(".config/wot/test_config.json");
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
            let file = File::create(path)?;
            serde_json::to_writer_pretty(file, &app).expect("Не смогли создать конфиг")
        }
    }
    Ok(())
}
