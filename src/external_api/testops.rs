use crate::errors::{WotApiError, WotError};
use crate::Config;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::multipart;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::error::Error;
use std::path::PathBuf;

#[derive(Debug, Clone)]
/// Асинхронный API клиент для TestOps
pub struct TestopsApiClient {
    headers: HeaderMap,
    pub base_url: String,
    client: reqwest::Client,
    postfix_after_base_url: String,
}

impl TestopsApiClient {
    /// Создание API клиента
    pub fn new(config: &Config) -> Self {
        let mut auth_header = HeaderMap::new();
        auth_header.insert(
            "Authorization",
            HeaderValue::from_str(format!("Api-Token {}", config.testops_api_token).as_str())
                .expect(&WotError::ParseHeaderValue.to_string()),
        );
        Self {
            headers: auth_header,
            base_url: config.testops_base_url.clone(),
            client: reqwest::Client::new(),
            postfix_after_base_url: "/api/rs".to_string(),
        }
    }

    /// get метод
    /// в url_method идет все, что после base_url и начинается с /
    async fn get(
        self,
        endpoint: String,
        headers: Option<HeaderMap>,
    ) -> Result<String, Box<dyn Error>> {
        let all_headers = match headers {
            Some(mut value) => {
                value.extend(self.headers);
                value
            }
            None => self.headers,
        };
        Ok(self
            .client
            .get(format!(
                "{}{}{}",
                self.base_url, self.postfix_after_base_url, endpoint
            ))
            .headers(all_headers)
            .send()
            .await?
            .text()
            .await?)
    }

    /// POST запрос, в котором можно отправить file
    async fn post_with_file(
        self,
        endpoint: String,
        multipart: multipart::Form,
        headers: Option<HeaderMap>,
    ) -> Result<String, Box<dyn Error>> {
        let all_headers = match headers {
            Some(mut value) => {
                value.extend(self.headers);
                value
            }
            None => self.headers,
        };
        Ok(self
            .client
            .post(format!(
                "{}{}{}",
                self.base_url, self.postfix_after_base_url, endpoint
            ))
            .headers(all_headers)
            .multipart(multipart)
            .send()
            .await?
            .text()
            .await?)
    }

    /// post метод
    /// в url_method идет все, что после base_url и начинается с /
    async fn post(
        self,
        endpoint: String,
        body: String,
        mut headers: HeaderMap,
    ) -> Result<String, Box<dyn Error>> {
        headers.extend(self.headers);
        Ok(self
            .client
            .post(format!(
                "{}{}{}",
                self.base_url, self.postfix_after_base_url, endpoint
            ))
            .headers(headers)
            .body(body)
            .send()
            .await?
            .text()
            .await?)
    }

    /// Получаем Part .zip архива для передачи в multipart::Form
    fn get_part_zip_archive(
        &self,
        full_file_path_to_archive: &PathBuf,
    ) -> Result<multipart::Part, Box<dyn Error>> {
        // Читаем массив байтов
        let file_to_bytes = std::fs::read(&full_file_path_to_archive).map_err(|e| {
            WotError::NotReadFile(
                full_file_path_to_archive.to_str().unwrap().to_string(),
                e.to_string(),
            )
        })?;
        // Проверяем, что расширение файла == zip
        let extension_file = full_file_path_to_archive
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");
        if extension_file != "zip" {
            return Err(WotError::ExtensionZip(extension_file.to_string()).into());
        };
        // Получаем имя файла из переданного пути
        let file_name = &full_file_path_to_archive
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| {
                WotError::NotFileName(full_file_path_to_archive.to_str().unwrap().to_string())
            })?;
        // Формируем и возвращаем Part для zip архива
        multipart::Part::bytes(file_to_bytes)
            .file_name(file_name.to_string())
            .mime_str("application/zip")
            .map_err(|e| WotApiError::Multipart(file_name.to_string(), e.to_string()).into())
    }

    pub async fn get_launch_by_id(
        self,
        launch_id: u32,
    ) -> Result<GetLauncByIdResponce, Box<dyn Error>> {
        let response = self.get(format!("/launch/{}", launch_id), None).await?;
        if response.is_empty() {
            return Err(WotApiError::EmptyResponse("/launch/[launch_id]".to_string()).into());
        }
        match serde_json::from_str(&response) {
            Ok(value) => Ok(value),
            Err(e) => Err(WotApiError::ParsingResponse(
                "/launch/[launch_id]".to_string(),
                e.to_string(),
            )
            .into()),
        }
    }

    /// Загрузка архива с отчетом в лаунч TestOps
    ///
    /// Параметры:
    ///
    /// full_file_path_to_archive: полный путь до архива с результатами тестов
    /// launch_info: структура LaunchInfo { name, project_id }
    pub async fn post_archive_report_launch_upload(
        self,
        full_file_path_to_archive: &PathBuf,
        launch_info: LaunchInfo,
    ) -> Result<ResponseLaunchUpload, Box<dyn Error>> {
        let file_part = self.get_part_zip_archive(full_file_path_to_archive)?;

        let info_file_json = serde_json::to_string(&launch_info)?;
        let info_file_multipart = multipart::Part::text(info_file_json)
            .mime_str("application/json")
            .unwrap();
        let multipart_form = multipart::Form::new()
            .part("info", info_file_multipart)
            .part("archive", file_part);
        let response = self
            .post_with_file("/launch/upload".to_string(), multipart_form, None)
            .await?;
        if response.is_empty() {
            return Err(WotApiError::EmptyResponse("/launch/upload".to_string()).into());
        }
        match serde_json::from_str(&response) {
            Ok(value) => Ok(value),
            Err(e) => Err(WotApiError::ParsingResponse(
                "/launch/upload".to_string(),
                e.to_string(),
            )
            .into()),
        }
    }

    /// Получение списка всех проектов
    pub async fn get_all_project_ids(self) -> Result<HashSet<u32>, Box<dyn Error>> {
        let mut current_page: u32 = 0;
        let limit_pages: u32 = 50;
        let mut project_ids: HashSet<u32> = HashSet::new();
        loop {
            let response = self
                .clone()
                .get(format!("/project?page={}", current_page), None)
                .await?;
            if response.is_empty() {
                return Err(WotApiError::EmptyResponse("/project".to_string()).into());
            }
            let resp_get_all_project =
                match serde_json::from_str::<ResponseGetAllProject>(&response) {
                    Ok(value) => value,
                    Err(e) => {
                        return Err(WotApiError::ParsingResponse(
                            "/project".to_string(),
                            e.to_string(),
                        )
                        .into())
                    }
                };
            if resp_get_all_project.total_pages < (current_page + 1) || current_page >= limit_pages
            {
                break;
            };
            project_ids.extend(
                resp_get_all_project
                    .content
                    .iter()
                    .map(|project_info| project_info.id),
            );
            current_page += 1;
        }
        Ok(project_ids)
    }

    /// Получение информации о проекте по его ID
    pub async fn get_project_info_by_id(
        self,
        project_id: &u32,
    ) -> Result<ProjectInfo, Box<dyn Error>> {
        let response = self.get(format!("/project/{}", project_id), None).await?;
        if response.is_empty() {
            return Err(WotApiError::EmptyResponse("/project/<id>".to_string()).into());
        }
        match serde_json::from_str::<ProjectInfo>(&response) {
            Ok(value) => Ok(value),
            Err(e) => {
                Err(WotApiError::ParsingResponse("/project/<id>".to_string(), e.to_string()).into())
            }
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ProjectInfo {
    pub id: u32,
    pub name: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ResponseGetAllProject {
    pub total_pages: u32,
    pub content: Vec<ProjectInfo>,
}
// https://serde.rs/enum-representations.html брал пример отсюда
#[derive(Deserialize, Serialize, Debug)]
// чтобы преобразовать camelCase в snake_case
#[serde(rename_all = "camelCase")]
pub struct GetLauncByIdResponce {
    id: u32,
    name: String,
    project_id: u32,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LaunchInfo {
    name: String,
    project_id: u32,
}

impl LaunchInfo {
    pub fn new(name: &str, project_id: u32) -> Self {
        Self {
            name: name.to_string(),
            project_id,
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ResponseLaunchUpload {
    #[serde(rename = "launchId")]
    pub launch_id: u32,
    #[serde(rename = "testSessionId")]
    test_session_id: u32,
    #[serde(rename = "filesCount")]
    files_count: u32,
}

#[cfg(test)]
mod tests {

    use super::*;
    use std::env;
    fn testops_api_client() -> TestopsApiClient {
        let config = Config {
            testops_base_url: env::var("TESTOPS_BASE_URL").unwrap(),
            testops_api_token: env::var("TESTOPS_API_TOKEN").unwrap(),
        };
        TestopsApiClient::new(&config)
    }

    #[tokio::test]
    /// Проверяем загрузку лаунча
    async fn test_upload_launch() {
        let launch_info = LaunchInfo {
            name: "check upload".to_string(),
            project_id: 2,
        };
        let path_archive = PathBuf::from(format!(
            "{}/test_files/testops_results_report_1735389182.zip",
            env!("CARGO_MANIFEST_DIR")
        ));
        let _ = testops_api_client()
            .post_archive_report_launch_upload(&path_archive, launch_info)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_get_all_project_ids() {
        let resp = testops_api_client().get_all_project_ids().await.unwrap();
        assert!(resp.contains(&2), "Не нашли проект с id == 2");
    }

    #[tokio::test]
    async fn test_get_project_by_id() {
        let resp = testops_api_client()
            .get_project_info_by_id(&2)
            .await
            .unwrap();
        assert_eq!("TestProject", resp.name);
    }

    #[test]
    /// Проверяем, что функция get_part_zip_archive отрабатывает
    /// для архива .zip
    fn test_get_part_zip_archive() {
        let file_path = PathBuf::from(format!(
            "{}/test_files/testops_results_report_1735389182.zip",
            env!("CARGO_MANIFEST_DIR")
        ));
        let _ = testops_api_client()
            .get_part_zip_archive(&file_path)
            .unwrap();
    }

    #[test]
    /// Проверяем, что функция get_part_zip_archive отдает ошибку для файла, у которого
    /// расширение НЕ .zip
    fn test_get_part_zip_archive_for_json() {
        let file_path = PathBuf::from(format!(
            "{}/test_files/file.json",
            env!("CARGO_MANIFEST_DIR")
        ));
        let exp_err = "Нужен файл с расширением .zip, был передан файл: *.json".to_string();
        let act_err = testops_api_client()
            .get_part_zip_archive(&file_path)
            .unwrap_err()
            .to_string();
        assert_eq!(act_err, exp_err, "Не получили ошибку для файла *.json");
    }

    #[test]
    /// Проверяем, что функция get_part_zip_archive отрабатывает для пустого файла .zip
    fn test_get_part_zip_archive_empty_file() {
        let file_path = PathBuf::from(format!(
            "{}/test_files/empty_files.zip",
            env!("CARGO_MANIFEST_DIR")
        ));
        let _ = testops_api_client()
            .get_part_zip_archive(&file_path)
            .unwrap();
    }

    #[test]
    /// Проверяем, что функция get_part_zip_archive обрабатывает ошибку, если передается директория
    fn test_get_part_zip_archive_for_dir() {
        let file_path = PathBuf::from(format!("{}/test_files/", env!("CARGO_MANIFEST_DIR")));
        let exp_err = "Не смогли прочитать файл по пути: \
            \"/Users/valentins/Desktop/rust_projects/plugin_testops/test_files/\". \
            Получили ошибку: Is a directory (os error 21)"
            .to_string();
        let act_err = testops_api_client()
            .get_part_zip_archive(&file_path)
            .unwrap_err()
            .to_string();
        assert_eq!(act_err, exp_err, "Не получили ошибку про директорию");
    }

    #[test]
    /// Проверяем, что функция get_part_zip_archive отрабатывает для пустого файла .zip
    fn test_get_part_zip_archive_empty_path() {
        let file_path = PathBuf::from("");
        let exp_err = "Не смогли прочитать файл по пути: \"\". \
            Получили ошибку: No such file or directory (os error 2)"
            .to_string();
        let act_err = testops_api_client()
            .get_part_zip_archive(&file_path)
            .unwrap_err()
            .to_string();
        assert_eq!(
            act_err, exp_err,
            "Не получили ошибку, что не смогли прочитать файл"
        );
    }
}
