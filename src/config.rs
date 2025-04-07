use crate::constants::{COMPLETE_SETUP, ENTER_INSTANCE_URL_TESTOPS, ENTER_TESTOPS_API_KEY};
use super::external_api::ApiError;
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
    /// Created app config
    pub fn new() -> Result<Self, ApiError> {
        println!("{}", ENTER_INSTANCE_URL_TESTOPS);
        let testops_base_url = validate_url(get_data_from_user_input()?)?;

        println!("{}", ENTER_TESTOPS_API_KEY);
        let testops_api_token = validate_testops_api_token(&get_data_from_user_input()?)?;
        println!("{}", COMPLETE_SETUP);

        Ok(Self {
            testops_base_url,
            testops_api_token: testops_api_token.trim().to_string(),
        })
    }

    /// Get data from app config
    pub fn get_config(path_to_config: PathBuf) -> Result<Self, ApiError> {
        let file = File::open(path_to_config)?;
        serde_json::from_reader(file).map_err(ApiError::Serde)
    }
}

fn validate_url(mut value: String) -> Result<String, ApiError> {
    let regex = Regex::new(r"^https?://.+$").unwrap();
    value.truncate(value.trim_end_matches('/').len());

    if !regex.is_match(&value) {
        return Err(ApiError::InvalidUrl)
    }
    Ok(value)
}

fn validate_testops_api_token(value: &str) -> Result<String, ApiError> {
    if Uuid::parse_str(value).is_err() {
        return Err(ApiError::InvalidToken);
    }
    Ok(value.to_string())
}

// todo поискать/посмотреть как правильно тестить такие штуки
fn get_data_from_user_input() -> Result<String, ApiError> {
    let mut input_value = String::new();
    io::stdin().read_line(&mut input_value)?;
    Ok(input_value.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case(String::from("http://instance.ru/api/rs"), String::from("http://instance.ru/api/rs"); "http")]
    #[test_case(String::from("https://example.com"), String::from("https://example.com"); "https")]
    #[test_case(String::from("https://instance.ru/"), String::from("https://instance.ru"); "last_char_slash")]
    #[test_case(String::from("https://example.com////"), String::from("https://example.com"); "multiple_slashes")]
    fn test_valid_url(url: String, exp_url: String) {
        let res = validate_url(url).unwrap();
        assert_eq!(res, exp_url, "Ожидали, что URL '{res}' пройдет валидацию");
    }

    #[test_case(String::from(""); "empty string")]
    #[test_case(String::from("htttp://google.com"); "invalid url")]
    fn test_invalid_url(url: String) {
        let res = validate_url(url).unwrap_err().to_string();
        assert_eq!(
            res, "The string entered must be a URL",
            "Ожидали что URL НЕ пройдет валидацию и мы получим сообщение об ошибке"
        );
    }

    #[test]
    fn test_testops_api_token_valid() {
        let uuid_token = Uuid::new_v4().to_string();
        let res = validate_testops_api_token(&uuid_token).unwrap();
        assert_eq!(
            res,
            uuid_token,
            "Ожидали, что api_token: {} пройдет валидацию",
            uuid_token
        );
    }

    #[test_case(""; "empty token")]
    #[test_case("c4e42f15-5b22-6ae-b2-10b5e2ffcb14"; "invalid uuid token")]
    fn test_testops_validate_api_token_invalid(value: &str) {
        let res = validate_testops_api_token(&value).unwrap_err().to_string();
        assert_eq!(res, "Your token failed validation, please try again",
            "Ожидали что api token: '{value}' НЕ пройдет валидацию и мы получим сообщение об ошибке");
    }

    #[test]
    fn test_get_config_by_invalid_path() {
        let path: PathBuf = PathBuf::from(format!(
            "{}/test_files/empty_config_for_test.json",
            env!("CARGO_MANIFEST_DIR")
        ));
        let error = Config::get_config(path).unwrap_err();
        assert_eq!("Deserialization error: missing field `testops_base_url` at line 1 column 2", error.to_string());
    }
}
