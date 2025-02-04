use crate::constants::{COMPLETE_SETUP, ENTER_INSTANCE_URL_TESTOPS, ENTER_TESTOPS_API_KEY};
use crate::errors::{WotError, COULD_READ_LINE};
use regex::Regex;
use serde::Deserialize;
use serde::Serialize;
use std::error::Error;
use std::fs::File;
use std::io;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Deserialize, Serialize, Debug)]
pub struct Config {
    /// Instance url
    pub testops_base_url: String,
    /// Token for authorization in TestOps API
    pub testops_api_token: String,
}

impl Config {
    /// Создаем конфиг приложения
    pub fn new() -> Result<Self, Box<dyn Error>> {
        println!("{}", ENTER_INSTANCE_URL_TESTOPS);
        let testops_base_url = validate_url(get_data_from_user_input())?;

        println!("{}", ENTER_TESTOPS_API_KEY);
        let testops_api_token = get_data_from_user_input();
        let _ = validate_testops_api_token(&testops_api_token)?;
        println!("{}", COMPLETE_SETUP);

        Ok(Self {
            testops_base_url,
            testops_api_token: testops_api_token.trim().to_string(),
        })
    }

    /// Получаем данные из конфига приложения
    ///
    /// Параметры:
    /// - path_to_config: путь до конифга приложения
    pub fn get_config(path_to_config: PathBuf) -> Result<Self, Box<dyn Error>> {
        let file = File::open(path_to_config)?;
        match serde_json::from_reader(file) {
            Ok(config) => Ok(config),
            Err(_) => Err(WotError::NotParseConfig.into()),
        }
    }
}

/// Введенная строка должна быть URL
fn validate_url(mut value: String) -> Result<String, WotError> {
    let regex = Regex::new(r"^https?://.+$").unwrap();
    if !regex.is_match(&value) {
        return Err(WotError::InvalidURL);
    }
    if value.ends_with('/') {
        value.pop();
    }
    Ok(value)
}

/// Валидация параметра testops_api_token
fn validate_testops_api_token(value: &str) -> Result<bool, WotError> {
    if Uuid::parse_str(value).is_err() {
        return Err(WotError::InvalidToken);
    }
    Ok(true)
}

fn get_data_from_user_input() -> String {
    let mut input_value = String::new();
    io::stdin()
        .read_line(&mut input_value)
        .expect(COULD_READ_LINE);
    input_value.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case(String::from("http://instance.ru/api/rs"), String::from("http://instance.ru/api/rs"); "http")]
    #[test_case(String::from("https://example.com"), String::from("https://example.com"); "https")]
    #[test_case(String::from("https://instance.ru/"), String::from("https://instance.ru"); "last_char_slash")]
    fn test_valid_url(url: String, exp_url: String) {
        let res = validate_url(url).unwrap();
        assert_eq!(res, exp_url, "Ожидали, что URL '{res}' пройдет валидацию");
    }

    #[test_case(String::from(""); "empty string")]
    #[test_case(String::from("htttp://google.com"); "invalid url")]
    fn test_invalid_url(url: String) {
        let res = validate_url(url).unwrap_err().to_string();
        assert_eq!(
            res, "Введенная строка должна быть URL",
            "Ожидали что URL НЕ пройдет валидацию и мы получим сообщение об ошибке"
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

    #[test]
    fn test_get_config_by_invalid_path() {
        let path: PathBuf = PathBuf::from(format!(
            "{}/test_files/empty_config_for_test.json",
            env!("CARGO_MANIFEST_DIR")
        ));
        let error = Config::get_config(path).unwrap_err();
        assert_eq!("Не смогли распарсить конфиг", error.to_string());
    }
}
