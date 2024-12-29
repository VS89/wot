use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::multipart;
use serde::{Deserialize, Serialize};
use std::env;
use std::error::Error;
use std::path::PathBuf;
use tokio::fs;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;

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
        archive_file_path: PathBuf,
    ) -> Result<multipart::Part, Box<dyn Error>> {
        // Читаем массив байтов
        let file_to_bytes = std::fs::read(&archive_file_path).map_err(|e| {
            format!(
                "Не смогли прочитать файл по пути: {:?}. Получили ошибку: {:?}",
                &archive_file_path, e
            )
        })?;
        // Проверяем, что расширение файла == zip
        let extension_file = archive_file_path
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
        let file_name = &archive_file_path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| {
                format!(
                    "Не удалось получить имя файла из пути: {:?}",
                    &archive_file_path
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
    pub async fn post_archive_report_launch_upload(
        self,
        archive_file_path: PathBuf,
        launch_info: LaunchInfo,
    ) -> Result<ResponseLaunchUpload, Box<dyn Error>> {
        let file_part = self.get_part_zip_archive(archive_file_path)?;

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
        // if response.is_empty() {
        //     todo!()
        // }
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

#[derive(Deserialize, Serialize, Debug)]
pub struct ResponseLaunchUpload {
    #[serde(rename = "launchId")]
    launch_id: u32,
    #[serde(rename = "testSessionId")]
    test_session_id: u32,
    #[serde(rename = "filesCount")]
    files_count: u32,
}

#[cfg(test)]
mod tests {
    use std::result;

    use reqwest::header::HeaderValue;
    use tokio::fs::OpenOptions;
    use tokio::io::AsyncWriteExt;

    use super::*;

    #[tokio::test]
    async fn test_upload_launch() {
        let testops_api_client = TestopsApiClient::new(env::var("TESTOPS_BASE_URL").unwrap());
        let launch_info = LaunchInfo {
            name: "check upload 2222".to_string(),
            project_id: 2,
        };
        let path_archive =
            PathBuf::from("/Users/valentins/Desktop/testops_results_report_1735389182.zip");
        let res = testops_api_client
            .post_archive_report_launch_upload(path_archive, launch_info)
            .await
            .unwrap();
        // Открытие файла для дозаписи (или создание, если он не существует)
        let mut file = OpenOptions::new()
            .append(true) // Режим дозаписи
            .create(true) // Cоздать файл, если он не существует
            .open("log.log")
            .await
            .unwrap();
        // Запись строки в файл
        file.write_all(serde_json::to_string(&res).unwrap().as_bytes())
            .await
            .unwrap();
    }

    #[tokio::test]
    /// Получение инфы по лаунчу
    async fn test_get_launch_by_id() {
        let testops_api_client = TestopsApiClient::new(env::var("TESTOPS_BASE_URL").unwrap());
        // todo надо нормально допилить этот момент и разобраться как структуры записывать в файл, чтобы дебажить
        let result = testops_api_client.get_launch_by_id(41213).await.unwrap();
        // Открытие файла для дозаписи (или создание, если он не существует)
        let mut file = OpenOptions::new()
            .append(true) // Режим дозаписи
            .create(true) // Cоздать файл, если он не существует
            .open("log.log")
            .await
            .unwrap();
        // Запись строки в файл
        file.write_all(serde_json::to_string(&result).unwrap().as_bytes())
            .await
            .unwrap();
    }
}
