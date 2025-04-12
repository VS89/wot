pub mod testops_api;

use reqwest::{header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, CONTENT_TYPE}, multipart, Client, StatusCode, Url};
use thiserror::Error;

const APPLICATION_JSON: &str = "application/json";

// todo по хорошему надо разделить ошибки от апи и от других мест
#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Network error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("Api error {0}: {1}")]
    Api(StatusCode, String),
    #[error("Deserialization error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("URL parse error: {0}")]
    Parse(String),
    #[error("Invalid API key")]
    InvalidApiKey,
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid file name")]
    InvalidFileName,
    #[error("Invalid file format: File is not a valid ZIP archive")]
    InvalidFileFormat,
    #[error("Project with ID == {0} not found")]
    ProjectIdNotFound(u32),
    #[error("Failed to retrieve the user's directories")]
    NotFoundUserDir,
    #[error("Invalid system time")]
    InvalidSystemTime,
    #[error("Could not find the directory at path: <{0}>")]
    NotFoundDirByPath(String),
    #[error("ZipError: {0}")]
    ZipError(#[from] zip::result::ZipError),
    // todo покрыть отсюда тестами
    #[error("The string entered must be a URL")]
    InvalidUrl,
    #[error("Your token failed validation, please try again")]
    InvalidToken,
    #[error("Could not create the file")]
    CouldNotCreateFile,
    #[error("Couldn't find a test case with ID == {0}")]
    CouldNotFindTestCaseById(u32),
    #[error("Couldn't create a config")]
    CantCreateConfig,
    #[error("The project ID must be greater than zero")]
    ProjectIdMoreThenZero,
    #[error("Upload cancelled by user")]
    UploadCancelledByUser
}

/// Basic api client
pub struct BaseApiClient {
    client: Client,
    pub base_url: Url,
}

impl BaseApiClient {

    fn build_url(&self, endpoint: &str) -> Result<Url, ApiError> {
        self.base_url.join(endpoint).map_err(|e| ApiError::Parse(e.to_string()))
    }

    fn get_default_headers(api_key: &str) -> Result<HeaderMap, ApiError>{
        let mut headers = HeaderMap::with_capacity(3);
        headers.insert(ACCEPT, HeaderValue::from_static(APPLICATION_JSON));
        headers.insert(CONTENT_TYPE, HeaderValue::from_static(APPLICATION_JSON));
        
        match HeaderValue::from_str(&format!("Api-Token {}", api_key)) {
            Ok(value) => headers.insert(AUTHORIZATION, value),
            Err(_) => return Err(ApiError::InvalidApiKey)
        };
        
        Ok(headers)
    }

    // todo надо покрыть тестами эту функцию
    async fn handle_response<T: serde::de::DeserializeOwned> (
        &self,
        response: reqwest::Response
    ) -> Result<T, ApiError> {
        let status = response.status();
        let body = response.text().await?;

        if !status.is_success() {
            return Err(ApiError::Api(status, body));
        }

        match serde_json::from_str(&body) {
            Ok(value) => Ok(value),
            Err(e) => Err(ApiError::Serde(e))
        }
    }

    pub fn new(base_url: &str, api_key: &str) -> Result<Self, ApiError> {
        let default_headers = Self::get_default_headers(api_key)?;

        let client = Client::builder()
            .default_headers(default_headers)
            .timeout(std::time::Duration::from_secs(10))
            .build()?;

        let parse_base_url = Url::parse(base_url)
            .map_err(|e| ApiError::Parse(e.to_string()))?;

        Ok(
            Self { 
                client, 
                base_url: parse_base_url, 
        })
    }

    pub async fn get<T: serde::de::DeserializeOwned, B: serde::Serialize>(
        &self,
        endpoint: &str,
    ) -> Result<T, ApiError> {
        let url = self.build_url(endpoint)?;
        let response= self.client.get(url).send().await?;
        self.handle_response(response).await
    }

    pub async fn post_multipart_file<T: serde::de::DeserializeOwned, B: serde::Serialize>(
        &self,
        endpoint: &str,
        multipart: multipart::Form,
    ) -> Result<T, ApiError> {
        let url = self.build_url(endpoint)?;
        let response = self.client.post(url).multipart(multipart).send().await?;
        self.handle_response(response).await
    }
}




#[cfg(test)]
mod tests {

    use std::env;
    use tokio::fs::File;
    use tokio::io::AsyncReadExt;
    use std::path::Path;
    use zip::ZipArchive;
    use reqwest::multipart::{Form, Part};
    use std::fs;

    use super::*;

    #[derive(serde::Deserialize)]
    struct Project {
        name: String,
    }

    #[derive(serde::Deserialize, serde::Serialize)]
    #[serde(rename_all = "camelCase")]
    struct LaunchInfo {
        name: String,
        project_id: u32,
    }

    #[derive(serde::Deserialize, serde::Serialize, Debug)]
    pub struct ResponseLaunchUpload {
        #[serde(rename = "launchId")]
        pub launch_id: u32,
        #[serde(rename = "testSessionId")]
        test_session_id: u32,
        #[serde(rename = "filesCount")]
        files_count: u32,
    }

    #[tokio::test]
    async fn test_upload_testops_report() {
        // Создаем структуру
        let base_url = env::var("TESTOPS_BASE_URL").unwrap();
        let api_key = env::var("TESTOPS_API_TOKEN").unwrap();
        let base_api_client = BaseApiClient::new(&base_url, &api_key).unwrap();

        // Читаем в буфер файл
        let file_path = Path::new("test_files/test_upload_launch_report.zip");
        let mut file = File::open(file_path).await.unwrap();
        let mut buffer = vec![];
        let _ = file.read_to_end(&mut buffer);

        // Создаем multipart с данными файла
        // todo: возможно получение имени файла стоит вынести в отдельную функцию
        let file_name = file_path.file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| ApiError::InvalidFileName).unwrap();
        let file_part = Part::bytes(buffer).file_name(file_name).mime_str("application/zip").unwrap();

        let info_file_json = serde_json::to_string(&LaunchInfo{name: "test_launch".to_string(), project_id: 2}).unwrap();
        let info_file_multipart = Part::text(info_file_json).mime_str("application/json").unwrap();

        // Собираем форму с файлом
        let form = Form::new().part("info", info_file_multipart).part("archive", file_part);
        let result = base_api_client.post_multipart_file::<ResponseLaunchUpload, ()>("/api/rs/launch/upload", form).await.unwrap();
        assert!(result.launch_id != 0);
    }

    #[test]
    fn test_create_base_api_client() {
        // Создаем структуру
        let base_url = env::var("TESTOPS_BASE_URL").unwrap();
        let api_key = env::var("TESTOPS_API_TOKEN").unwrap();
        let base_api_client = BaseApiClient::new(&base_url, &api_key).unwrap();

        // assert!(!base_api_client.api_key.is_empty());
        assert!(!base_api_client.base_url.as_str().is_empty());
    }

    #[tokio::test]
    async fn test_get_request() {
        // Создаем структуру
        let base_url = env::var("TESTOPS_BASE_URL").unwrap();
        let api_key = env::var("TESTOPS_API_TOKEN").unwrap();
        let base_api_client = BaseApiClient::new(&base_url, &api_key).unwrap();

        let project: Project = base_api_client.get::<Project, ()>("/api/rs/project/2").await.unwrap();
        assert!(!project.name.is_empty());
    }

    #[tokio::test]
    async fn test_get_request_nonexists_project() {
        // Создаем структуру
        let base_url = env::var("TESTOPS_BASE_URL").unwrap();
        let api_key = env::var("TESTOPS_API_TOKEN").unwrap();
        let base_api_client = BaseApiClient::new(&base_url, &api_key).unwrap();

        let result = base_api_client.get::<(), ()>("/api/rs/project/9999").await.unwrap_err();
        let api_error = ApiError::from(result);
        assert!(matches!(api_error, ApiError::Api(_, _)));
        assert!(format!("{}", api_error).starts_with("Api error"));
    }

    #[test]
    fn test_build_url_positive(){
        // Создаем структуру
        let base_url = env::var("TESTOPS_BASE_URL").unwrap();
        let api_key = env::var("TESTOPS_API_TOKEN").unwrap();
        let base_api_client = BaseApiClient::new(&base_url, &api_key).unwrap();

        // Получаем url
        let new_url = base_api_client.build_url("/new_url").unwrap().to_string();

        // Проверяем, что получили правильный урл
        let exp_url = format!("{}{}", base_url, "/new_url");
        assert_eq!(exp_url, new_url, "{}", format!("Ожидали URL == {}. Получили == {}", exp_url, new_url));
    }

    #[test]
    fn test_valid_headers() {
        // Получаем хедеры
        let default_headers = BaseApiClient::get_default_headers("test_api_key").unwrap();

        // Проверяем значение дефолтных хедеров
        assert_eq!(default_headers[ACCEPT], "application/json");
        assert_eq!(default_headers[CONTENT_TYPE], "application/json");
        assert_eq!(default_headers[AUTHORIZATION], "Api-Token test_api_key");
    }

    #[test]
    fn test_invalid_api_key() {
        let api_key_err = BaseApiClient::get_default_headers("invalid\nkey").unwrap_err();
        assert!(
            matches!(api_key_err, ApiError::InvalidApiKey), 
            "Ожидали ошибку ApiError::InvalidApiKey, получили другую"
        );
    }

    #[test]
    fn test_url_parse_error() {
        // Получаем ошибку парсинга URL
        let url_parse_err = Url::parse("input").unwrap_err();

        // Конвертируем в ApiError
        let api_error = ApiError::Parse(url_parse_err.to_string());

        assert!(matches!(api_error, ApiError::Parse(_)));
        assert!(
            format!("{}", api_error).starts_with("URL parse error: "),
            "{}", format!("Ожидали, что ошибка будет начинаться с 'URL parse error: '. Получили {}", api_error.to_string())
        );
    }

    #[tokio::test]
    async fn test_reqwest_error() {
        // Создаем ошибку reqwest с невалдиным URL
        let req_client = reqwest::Client::builder().build().unwrap();
        let err_reqwest = req_client.get("https://").send().await.unwrap_err();

        // Конвертируем в ApiError
        let api_error = ApiError::from(err_reqwest);

        // Проверяем тип ошибки и сообщение 
        assert!(
            matches!(api_error, ApiError::Reqwest(_)), 
            "Получили не тот тип ошибки, который ожидали."
        );
        assert!(
            format!("{}", api_error).starts_with("Network error: "), 
            "{}", format!("Сообщение об ошибке должно начинаться с 'Network error: '. Получили ошибку: {}", api_error)
        );
    }

    #[tokio::test]
    async fn test_api_error() {
        // Создаем ApiError
        let api_error = ApiError::Api(reqwest::StatusCode::CREATED, String::from("test error"));
        let string_api_error = api_error.to_string();
        let exp_api_error_txt = String::from("Api error 201 Created: test error");

        assert!(
            matches!(api_error, ApiError::Api(_, _)),
            "Получили не тот тип ошибки, который ожидали."
        );
        assert_eq!(
            exp_api_error_txt, string_api_error,
            "{}", format!("Ожидали ошибку: {}, получили: {}", exp_api_error_txt, string_api_error)
        )
    }

    #[tokio::test]
    async fn test_serde_error() {
        // Пытаемся десериализовать невалидный json
        let serde_error = serde_json::from_str::<serde_json::Value>("invalid").unwrap_err();

        // Конвертируем в ApiError
        let api_error = ApiError::Serde(serde_error);

        // Проверяем тип ошибки и сообщение
        assert!(matches!(api_error, ApiError::Serde(_)), "Получили не тот тип ошибки, который ожидали");
        assert!(
            format!("{}", api_error).starts_with("Deserialization error: "),
            "{}", format!("Ожидали, что ошибка будет начинаться с 'Deserialization error: ', получили {}", api_error.to_string())
        );
    }

    #[test]
    fn test_zip_error_handling() {
        // Данные, которые не соответствуют формату ZIP
        let data = b"this is data";
        let reader = std::io::Cursor::new(&data[..]);

        // Пытаемся создать архив из невалидного буфера
        let result = ZipArchive::new(reader).map_err(ApiError::from);
        assert!(result.is_err());
        assert!(matches!(result, Err(ApiError::ZipError(_))));
    }

    #[test]
    fn test_not_found_dir_error() {
        let result = fs::read_dir("non/existent/path")
            .map_err(|_| ApiError::NotFoundDirByPath("non/existent/path".to_string()));
        
        assert!(matches!(result, Err(ApiError::NotFoundDirByPath(_))));
        assert_eq!(
            result.unwrap_err().to_string(),
            "Could not find the directory at path: <non/existent/path>"
        );
    }

    #[test]
    fn test_invalid_url() {
        assert_eq!(ApiError::InvalidUrl.to_string(), "The string entered must be a URL".to_string());
    }

    #[test]
    fn test_invalid_token() {
        assert_eq!(ApiError::InvalidToken.to_string(), "Your token failed validation, please try again".to_string());
    }

    #[test]
    fn test_could_not_create_file() {
        assert_eq!(ApiError::CouldNotCreateFile.to_string(), "Could not create the file".to_string());
    }

    #[test]
    fn test_could_not_find_test_case_by_id() {
        assert_eq!(ApiError::CouldNotFindTestCaseById(12345).to_string(), 
        format!("Couldn't find a test case with ID == {}", 12345).to_string());
    }

    #[test]
    fn test_cant_craete_config() {
        assert_eq!(ApiError::CantCreateConfig.to_string(), "Couldn't create a config".to_string());
    }

    #[test]
    fn test_project_id_more_then_zero() {
        assert_eq!(ApiError::ProjectIdMoreThenZero.to_string(), "The project ID must be greater than zero".to_string());
    }

    #[test]
    fn test_upload_cancelled_by_user() {
        assert_eq!(ApiError::UploadCancelledByUser.to_string(), "Upload cancelled by user".to_string());
    }
}