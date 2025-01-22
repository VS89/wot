use regex::Regex;
use serde::Deserialize;
use serde::Serialize;
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
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        println!("Enter the url of the testops instance: ");
        let testops_base_url = validate_url(get_data_from_user_input())?;

        println!("Enter the TestOps API key");
        let testops_api_token = get_data_from_user_input();
        let _ = validate_testops_api_token(&testops_api_token);
        println!("To view the available commands, type: wot --help");

        Ok(Self {
            testops_base_url,
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
fn validate_url(mut value: String) -> Result<String, Box<dyn std::error::Error>> {
    let regex = Regex::new(r"^https?://.+$").unwrap();
    if !regex.is_match(&value) {
        return Err("Введенная строка должна быть URL".into());
    }
    if value.chars().last().unwrap() == '/' {
        value.pop();
    }
    Ok(value)
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
    input_value.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    // todo видимо в main нельзя хранить тесты они почему то не запускаются если их пачкой запускать
    // или через cargo test

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
}
