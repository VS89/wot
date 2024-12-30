use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::multipart;
use serde::{Deserialize, Serialize};
use std::env;
use std::error::Error;
use std::path::PathBuf;

#[derive(Debug)]
/// Асинхронный API клиент для TestOps
pub struct TestopsApiClient {
    headers: HeaderMap,
    pub base_url: String,
    client: reqwest::Client,
}

impl TestopsApiClient {
    // Обновляем хедеры
    // todo надо подумать как правильно сделать эту функцию
    // fn update_headers(&self, headers: Option<HeaderMap>) -> HeaderMap {
    //     match headers {
    //         Some(mut value) => {
    //             value.extend(&self.headers);
    //             value
    //         }
    //         None => self.headers,
    //     }
    // }

    /// Создание API клиента
    pub fn new(base_url: String) -> Self {
        let api_token = env::var("TESTOPS_API_TOKEN").unwrap();
        let mut auth_header = HeaderMap::new();
        auth_header.insert(
            "Authorization",
            HeaderValue::from_str(format!("Api-Token {}", api_token).as_str())
                .expect("Не смогли преобразовать HeaderValue"),
        );
        Self {
            headers: auth_header,
            base_url,
            client: reqwest::Client::new(),
        }
    }

    /// get метод
    /// в url_method идет все, что после base_url и начинается с /
    async fn get(
        self,
        endpoint: String,
        headers: Option<HeaderMap>,
    ) -> Result<String, Box<dyn Error>> {
        // todo надо подумать как вынести это в отдельную функцию
        let all_headers = match headers {
            Some(mut value) => {
                value.extend(self.headers);
                value
            }
            None => self.headers,
        };
        Ok(self
            .client
            .get(format!("{}{}", self.base_url, endpoint))
            .headers(all_headers)
            .send()
            .await?
            .text()
            .await?)
    }

    // todo по хорошему наверно это надо объеденить с обычным post методом
    /// POST запрос, в котором можно отправить file
    async fn post_with_file(
        self,
        endpoint: String,
        multipart: multipart::Form,
        headers: Option<HeaderMap>,
    ) -> Result<String, Box<dyn Error>> {
        // todo надо подумать как вынести это в отдельную функцию
        let all_headers = match headers {
            Some(mut value) => {
                value.extend(self.headers);
                value
            }
            None => self.headers,
        };
        Ok(self
            .client
            .post(format!("{}{}", self.base_url, endpoint))
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
            .post(format!("{}{}", self.base_url, endpoint))
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
            format!(
                "Не смогли прочитать файл по пути: {:?}. Получили ошибку: {:?}",
                &full_file_path_to_archive, e
            )
        })?;
        // Проверяем, что расширение файла == zip
        let extension_file = full_file_path_to_archive
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");
        if extension_file != "zip" {
            return Err(format!(
                "Нужен файл с расширением .zip, был передан файл: *.{:?}",
                extension_file
            )
            .into());
        };
        // Получаем имя файла из переданного пути
        let file_name = &full_file_path_to_archive
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| {
                format!(
                    "Не удалось получить имя файла из пути: {:?}",
                    &full_file_path_to_archive
                )
            })?;
        // Формируем и возвращаем Part для zip архива
        multipart::Part::bytes(file_to_bytes)
            .file_name(file_name.to_string())
            .mime_str("application/zip")
            .map_err(|e| {
                format!(
                    "Не смогли преобразовать файл {:?} для API-запроса. Ошибка: {:?}",
                    file_name, e
                )
                .into()
            })
    }

    // todo надо нормально допилить этот момент и разобраться как структуры записывать в файл, чтобы дебажить
    pub async fn get_launch_by_id(
        self,
        launch_id: u32,
    ) -> Result<GetLauncByIdResponce, Box<dyn Error>> {
        let response = self.get(format!("/launch/{}", launch_id), None).await?;
        if response.is_empty() {
            return Err("Получили пустой ответ от метода /launch/[launch_id]".into());
        }
        match serde_json::from_str(&response) {
            Ok(value) => Ok(value),
            Err(e) => Err(format!(
                "При парсинге ответа метода /launch/[launch_id] получили ошибку: {}",
                e
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
            return Err("Получили пустой ответ в ответе метода /launch/upload".into());
        }
        match serde_json::from_str(&response) {
            Ok(value) => Ok(value),
            Err(e) => Err(format!(
                "При парсинге ответа метода /launch/upload получили ошибку {}",
                e
            )
            .into()),
        }
    }
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
// {"launchId":42369,"testSessionId":112014,"filesCount":56}

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

    #[tokio::test]
    /// Проверяем загрузку лаунча
    async fn test_upload_launch() {
        let testops_api_client = TestopsApiClient::new(env::var("TESTOPS_BASE_API_URL").unwrap());
        let launch_info = LaunchInfo {
            name: "check upload".to_string(),
            project_id: 2,
        };
        let path_archive = PathBuf::from(format!(
            "{}/test_files/testops_results_report_1735389182.zip",
            env!("CARGO_MANIFEST_DIR")
        ));
        let _ = testops_api_client
            .post_archive_report_launch_upload(path_archive, launch_info)
            .await
            .unwrap();
    }

    #[test]
    /// Проверяем, что функция get_part_zip_archive отрабатывает
    /// для архива .zip
    fn test_get_part_zip_archive() {
        let testops_api_client = TestopsApiClient::new(env::var("TESTOPS_BASE_API_URL").unwrap());
        let file_path = PathBuf::from(format!(
            "{}/test_files/testops_results_report_1735389182.zip",
            env!("CARGO_MANIFEST_DIR")
        ));
        let _ = testops_api_client.get_part_zip_archive(file_path).unwrap();
    }

    #[test]
    /// Проверяем, что функция get_part_zip_archive отдает ошибку для файла, у которого
    /// расширение НЕ .zip
    fn test_get_part_zip_archive_for_json() {
        let testops_api_client = TestopsApiClient::new(env::var("TESTOPS_BASE_API_URL").unwrap());
        let file_path = PathBuf::from(format!(
            "{}/test_files/file.json",
            env!("CARGO_MANIFEST_DIR")
        ));
        let exp_err = "Нужен файл с расширением .zip, был передан файл: *.\"json\"".to_string();
        let act_err = testops_api_client
            .get_part_zip_archive(file_path)
            .unwrap_err()
            .to_string();
        assert_eq!(act_err, exp_err, "Не получили ошибку для файла *.json");
    }

    #[test]
    /// Проверяем, что функция get_part_zip_archive отрабатывает для пустого файла .zip
    fn test_get_part_zip_archive_empty_file() {
        let testops_api_client = TestopsApiClient::new(env::var("TESTOPS_BASE_API_URL").unwrap());
        let file_path = PathBuf::from(format!(
            "{}/test_files/empty_files.zip",
            env!("CARGO_MANIFEST_DIR")
        ));
        let _ = testops_api_client.get_part_zip_archive(file_path).unwrap();
    }

    #[test]
    /// Проверяем, что функция get_part_zip_archive обрабатывает ошибку, если передается директория
    fn test_get_part_zip_archive_for_dir() {
        let testops_api_client = TestopsApiClient::new(env::var("TESTOPS_BASE_API_URL").unwrap());
        let file_path = PathBuf::from(format!("{}/test_files/", env!("CARGO_MANIFEST_DIR")));
        let exp_err = "Не смогли прочитать файл по пути: \
            \"/Users/valentins/Desktop/rust_projects/plugin_testops/test_files/\". \
            Получили ошибку: Os { code: 21, kind: IsADirectory, message: \"Is a directory\" }"
            .to_string();
        let act_err = testops_api_client
            .get_part_zip_archive(file_path)
            .unwrap_err()
            .to_string();
        assert_eq!(act_err, exp_err, "Не получили ошибку про директорию");
    }

    #[test]
    /// Проверяем, что функция get_part_zip_archive отрабатывает для пустого файла .zip
    fn test_get_part_zip_archive_empty_path() {
        let testops_api_client = TestopsApiClient::new(env::var("TESTOPS_BASE_API_URL").unwrap());
        let file_path = PathBuf::from("");
        let exp_err = "Не смогли прочитать файл по пути: \"\". \
            Получили ошибку: Os { code: 2, kind: NotFound, message: \"No such file or directory\" }"
            .to_string();
        let act_err = testops_api_client
            .get_part_zip_archive(file_path)
            .unwrap_err()
            .to_string();
        assert_eq!(
            act_err, exp_err,
            "Не получили ошибку, что не смогли прочитать файл"
        );
    }
}
