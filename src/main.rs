use clap::{Args, Parser, Subcommand};
use std::env;
use std::fs;
use tokio;
use wot::external_api::testops::{LaunchInfo, ResponseLaunchUpload, TestopsApiClient};
use wot::zip_directory;
// https://github.com/clap-rs/clap?tab=readme-ov-file
// https://docs.rs/clap/latest/clap/

/// Отправка отчета в TestOps
async fn send_report(path_report: &str, project_id: u32) -> Result<(), Box<dyn std::error::Error>> {
    let result = zip_directory(path_report)?;
    println!("{:?}", result);
    let testops = TestopsApiClient::new(env::var("TESTOPS_BASE_API_URL")?);
    let generate_launch_name = chrono::Local::now().format("%d/%m/%Y %H:%M").to_string();
    let launch_info = LaunchInfo::new(&format!("Запуск от {}", generate_launch_name), project_id);
    let response: ResponseLaunchUpload = testops
        .post_archive_report_launch_upload(&result, launch_info)
        .await
        .unwrap();
    let base_url_for_launch = env::var("TESTOPS_BASE_URL")?;
    println!(
        "Ссылка на загруженный лаунч: {}/launch/{}",
        base_url_for_launch, response.launch_id
    );
    let _ = fs::remove_file(&result);
    Ok(())
}

#[derive(Parser)]
#[command(
    name = "wot",
    version = "0.1.0",
    author = "Valentin Semenov <valentin@semenov-aqa.ru>",
    about = "Плагин для Allure TestOps <https://qameta.io/>. wot - WrapperOverTestops",
    long_about = None,
    propagate_version = true
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Report(ReportArgs),
}

#[derive(Args)]
struct ReportArgs {
    /// Директория
    #[arg(long, short, required = true)]
    directory_path: String,
    /// ID проекта
    #[arg(long, short, required = true)]
    project_id: u32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Report(value) => send_report(&value.directory_path, value.project_id).await?,
    }
    Ok(())
}
// /Users/valentins/Desktop/test_allure_report
