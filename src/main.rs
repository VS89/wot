use clap::{Args, Parser, Subcommand};
use directories::UserDirs;
use regex::Regex;
use serde::Deserialize;
use serde::Serialize;
use std::env;
use std::fs;
use std::fs::File;
use std::io;
use std::path::PathBuf;
use tokio;
use uuid::Uuid;
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
    /// Загрузка отчета в тестопс
    Report(ReportArgs),
}

#[derive(Args)]
struct ReportArgs {
    /// Директория
    #[arg(long, short, required = true)]
    directory_path: String,
    /// ID проекта
    #[arg(long, short, required = true, value_parser = validate_project_id)]
    project_id: u32,
}

#[derive(Deserialize, Serialize, Debug)]
/// Данные, которые нужны для корректной работы приложения
struct Application {
    /// Базовый API url, чаще всего http://<host_url>/api/rs
    testops_base_api_url: String,
    /// HOST url(может отличаться от host_url для API)
    testops_base_url: String,
    /// Токен для авторизации в API TestOps
    testops_api_token: String,
}

impl Application {
    /// Создаем конфиг приложения
    fn new() -> Self {
        // todo тут нужно проверить что нету конфига с заполненными данными
        // Если он есть и все данные заполнены, то скипать этот шаг
        //
        // todo наверно сюда стоит прикрутить все-таки какую-то базовую валидацию
        //
        // todo и надо будет вынести это все в отдельный файлик
        //
        // todo думаю надо будет прям тут создать 3 приватных функцию на 3 параметра
        // конфига
        println!("Введите url для testops API(пример: http://<url>/api/rs):");
        let mut testops_base_api_url = String::new();
        io::stdin()
            .read_line(&mut testops_base_api_url)
            .expect("Не смогли прочитать строку");
        println!("Введите host url testops(где отображается список проектов)");
        let mut testops_base_url = String::new();
        io::stdin()
            .read_line(&mut testops_base_url)
            .expect("Не смогли прочитать строку");
        println!("Введите testops API ключ");
        let mut testops_api_token = String::new();
        io::stdin()
            .read_line(&mut testops_api_token)
            .expect("Не смогли прочитать строку");
        println!("Для просмотра доступных команд введите: wot --help");
        Self {
            testops_base_api_url: testops_base_api_url.trim().to_string(),
            testops_base_url: testops_base_url.trim().to_string(),
            testops_api_token: testops_api_token.trim().to_string(),
        }
    }

    /// Введенная строка должна быть URL
    fn validate_url(value: &str) -> Result<bool, Box<dyn std::error::Error>> {
        let regex = Regex::new(r"^https?://.+$").unwrap();
        if !regex.is_match(value) {
            return Err("Введенная строка должна быть URL".into());
        }
        Ok(true)
    }

    /// Валидация параметра testops_api_token
    fn validate_testops_api_token(value: &str) -> Result<bool, Box<dyn std::error::Error>> {
        if !Uuid::parse_str(value).is_ok() {
            return Err("Ваш токен не прошел валидацию, попробуйте еще раз".into());
        }
        Ok(true)
    }

    /// Получаем данные из конфига приложения
    ///
    /// Параметры:
    /// - path_to_config: путь до конифга приложения
    fn get_from_config(path_to_config: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let file = File::open(path_to_config)?;
        let config: Self = serde_json::from_reader(file)
            .expect("Получили ошибку при чтение файла по пути: {path_to_config}");
        Ok(config)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    if let Some(user_dirs) = UserDirs::new() {
        let path = user_dirs.home_dir().join(".config/wot/config1122.json");
        if path.exists() {
            let _ = Application::get_from_config(path)?;
            let cli = Cli::parse();

            match &cli.command {
                Commands::Report(value) => {
                    send_report(&value.directory_path, value.project_id).await?
                }
            }
        } else {
            let app = Application::new();
            let file = File::create(path)?;
            serde_json::to_writer_pretty(file, &app).expect("Не смогли создать конфиг")
        }
    }
    Ok(())
}
// /Users/valentins/Desktop/test_allure_report

// Валидация параметра project_id. В целом простую валидацю параметра описывать не обязательно,
// ее отлично выполняет clap
// Надо бы еще прикрутить, чтобы тут брались id проектов из тестопса и показывало какие данные можно вводить
fn validate_project_id(value: &str) -> Result<u32, String> {
    let project_id: u32 = value
        .parse()
        .map_err(|_| format!("project_id должен быть целым числом > 0"))?;
    Ok(project_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    // todo видимо в main нельзя хранить тесты они почему то не запускаются если их пачкой запускать
    // или через cargo test

    /// Валидный URL
    #[test_case("http://some_domen.ru/api/rs"; "http")]
    #[test_case("https://example.com"; "https")]
    fn test_valid_url(url: &str) {
        let res = Application::validate_url(url).unwrap();
        assert!(res, "Ожидали, что URL '{url}' пройдет валидацию");
    }

    /// НЕвалидный URL
    #[test_case(""; "empty string")]
    #[test_case("htttp://google.com"; "invalid url")]
    fn test_invalid_url(url: &str) {
        let res = Application::validate_url(url).unwrap_err().to_string();
        assert_eq!(
            res, "Введенная строка должна быть URL",
            "Ожидали что URL: '{url}' НЕ пройдет валидацию и мы получим сообщение об ошибке"
        );
    }

    /// Валидный testops api token
    #[test]
    fn test_testops_api_token_valid() {
        let uuid_token = Uuid::new_v4().to_string();
        let res = Application::validate_testops_api_token(&uuid_token).unwrap();
        assert!(
            res,
            "Ожидали, что api_token: {} пройдет валидацию",
            uuid_token
        );
    }

    /// НЕвалидный testops api token
    #[test_case(""; "empty token")]
    #[test_case("c4e42f15-5b22-6ae-b2-10b5e2ffcb14"; "invalid uuid token")]
    fn test_testops_validate_api_token_invalid(value: &str) {
        let res = Application::validate_testops_api_token(&value)
            .unwrap_err()
            .to_string();
        assert_eq!(res, "Ваш токен не прошел валидацию, попробуйте еще раз",
            "Ожидали что api token: '{value}' НЕ пройдет валидацию и мы получим сообщение об ошибке");
    }
}
