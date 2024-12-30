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
#[tokio::main]
async fn main() {
    let result = zip_directory("/Users/valentins/Desktop/test_allure_report").unwrap();
    println!("{:?}", result);
    let testops = TestopsApiClient::new(env::var("TESTOPS_BASE_API_URL").unwrap());
    let launch_info = LaunchInfo::new("check work script", 2);
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
