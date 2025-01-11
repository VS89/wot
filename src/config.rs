use regex::Regex;
use serde::Deserialize;
use serde::Serialize;
use std::fs::File;
use std::io;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Deserialize, Serialize, Debug)]
pub struct Config {
    /// Base api url, example: http://<host_url>/api/rs
    pub testops_base_api_url: String,
    /// Host url
    pub testops_base_url: String,
    /// Token for authorization in TestOps API
    pub testops_api_token: String,
}

impl Config {
    /// Создаем конфиг приложения
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // todo По-идее это можно вынести в какую-нибудь отдельную функцию
        println!("Enter the url for the testops API (example: http://<url>/api/rs):");
        let testops_base_api_url = get_data_from_user_input();
        let _ = validate_url(&testops_base_api_url)?;

        println!(
            "Enter the host url TestOps (url that leads to the page with the list of projects)"
        );
        let testops_base_url = get_data_from_user_input();
        let _ = validate_url(&testops_base_url)?;

        println!("Enter the TestOps API key");
        let testops_api_token = get_data_from_user_input();
        let _ = validate_testops_api_token(&testops_api_token);
        println!("To view the available commands, type: wot --help");

        Ok(Self {
            testops_base_api_url: testops_base_api_url.trim().to_string(),
            testops_base_url: testops_base_url.trim().to_string(),
            testops_api_token: testops_api_token.trim().to_string(),
        })
    }

    /// Получаем данные из конфига приложения
    ///
    /// Параметры:
    /// - path_to_config: путь до конифга приложения
    pub fn get_config(path_to_config: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let file = File::open(path_to_config)?;
        let config: Self = serde_json::from_reader(file)
            .expect("Получили ошибку при чтение файла по пути: {path_to_config}");
        Ok(config)
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

fn get_data_from_user_input() -> String {
    let mut input_value = String::new();
    io::stdin()
        .read_line(&mut input_value)
        .expect("Couldn't read the line");
    input_value
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    // todo видимо в main нельзя хранить тесты они почему то не запускаются если их пачкой запускать
    // или через cargo test

    #[test_case("http://some_domen.ru/api/rs"; "http")]
    #[test_case("https://example.com"; "https")]
    fn test_valid_url(url: &str) {
        let res = validate_url(url).unwrap();
        assert!(res, "Ожидали, что URL '{url}' пройдет валидацию");
    }

    #[test_case(""; "empty string")]
    #[test_case("htttp://google.com"; "invalid url")]
    fn test_invalid_url(url: &str) {
        let res = validate_url(url).unwrap_err().to_string();
        assert_eq!(
            res, "Введенная строка должна быть URL",
            "Ожидали что URL: '{url}' НЕ пройдет валидацию и мы получим сообщение об ошибке"
        );
    }

    #[test]
    fn test_testops_api_token_valid() {
        let uuid_token = Uuid::new_v4().to_string();
        let res = validate_testops_api_token(&uuid_token).unwrap();
        assert!(
            res,
            "Ожидали, что api_token: {} пройдет валидацию",
            uuid_token
        );
    }

    #[test_case(""; "empty token")]
    #[test_case("c4e42f15-5b22-6ae-b2-10b5e2ffcb14"; "invalid uuid token")]
    fn test_testops_validate_api_token_invalid(value: &str) {
        let res = validate_testops_api_token(&value).unwrap_err().to_string();
        assert_eq!(res, "Ваш токен не прошел валидацию, попробуйте еще раз",
            "Ожидали что api token: '{value}' НЕ пройдет валидацию и мы получим сообщение об ошибке");
    }
}
