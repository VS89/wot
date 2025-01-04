use clap::Parser;
use std::env;
use std::fs;
use tokio;
use wot::external_api::testops::{LaunchInfo, ResponseLaunchUpload, TestopsApiClient};
use wot::zip_directory;
// https://github.com/clap-rs/clap?tab=readme-ov-file
// https://docs.rs/clap/latest/clap/

// Рабочий пример кейса:
// - Архивируем папку с результатами в .zip
// - Загружаем отчет через апи в тестопс
// - Получаем ссылку на лаунч
// - Удаляем архив
// #[tokio::main]
// async fn main() {
//     let result = zip_directory("/Users/valentins/Desktop/test_allure_report").unwrap();
//     println!("{:?}", result);
//     let testops = TestopsApiClient::new(env::var("TESTOPS_BASE_API_URL").unwrap());
//     let launch_info = LaunchInfo::new("check work script", 2);
//     let response: ResponseLaunchUpload = testops
//         .post_archive_report_launch_upload(&result, launch_info)
//         .await
//         .unwrap();
//     let base_url_for_launch = env::var("TESTOPS_BASE_URL").unwrap();
//     println!(
//         "Ссылка на загруженный лаунч: {}/launch/{}",
//         base_url_for_launch, response.launch_id
//     );
//     let _ = fs::remove_file(&result);
// }

// const CONFIG_FILE: &Path
//
/// Отправка отчета в TestOps
async fn send_report(path_report: &str, project_id: u32) {
    let result = zip_directory(path_report).unwrap();
    println!("{:?}", result);
    let testops = TestopsApiClient::new(env::var("TESTOPS_BASE_API_URL").unwrap());
    let launch_info = LaunchInfo::new("check work script", project_id);
    let response: ResponseLaunchUpload = testops
        .post_archive_report_launch_upload(&result, launch_info)
        .await
        .unwrap();
    let base_url_for_launch = env::var("TESTOPS_BASE_URL").unwrap();
    println!(
        "Ссылка на загруженный лаунч: {}/launch/{}",
        base_url_for_launch, response.launch_id
    );
    let _ = fs::remove_file(&result);
}

#[derive(Parser)]
#[command(name = "MyApp")]
#[command(version = "1.0")]
#[command(about = "Plugin for Allure TestOps <https://qameta.io/>. wot - WrapperOverTestops", long_about = None)]
struct Cli {
    #[arg(short, long)]
    send_report: String,
    #[arg(short, long)]
    project_id: u32,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    send_report(&cli.send_report, cli.project_id).await;
}
// /Users/valentins/Desktop/test_allure_report
