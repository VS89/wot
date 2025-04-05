use crate::errors::{WotApiError, WotError, PARSE_HEADER_VALUE};
use crate::Config;
use crate::create_file_in_current_directory;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::multipart;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::path::PathBuf;

#[derive(Debug, Clone)]
/// TestOps Async API client
pub struct TestopsApiClient {
    headers: HeaderMap,
    pub base_url: String,
    client: reqwest::Client,
    postfix_after_base_url: String,
}

impl TestopsApiClient {
    /// Created API client
    pub fn new(config: &Config) -> Self {
        let mut auth_header = HeaderMap::new();
        auth_header.insert(
            "Authorization",
            HeaderValue::from_str(format!("Api-Token {}", config.testops_api_token).as_str())
                .expect(PARSE_HEADER_VALUE),
        );
        Self {
            headers: auth_header,
            base_url: config.testops_base_url.clone(),
            client: reqwest::Client::new(),
            postfix_after_base_url: "/api/rs".to_string(),
        }
    }

    fn merge_headers(&self, headers: Option<HeaderMap>) -> HeaderMap {
        match headers {
            Some(mut value) => {
                value.extend(self.headers.clone());
                value
            }
            None => self.headers.clone()
        }
    }

    /// GET method
    async fn get(
        &self,
        endpoint: String,
        headers: Option<HeaderMap>,
    ) -> Result<String, Box<dyn Error>> {
        let all_headers = self.merge_headers(headers);
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

    /// POST method for work with file
    async fn post_with_file(
        &self,
        endpoint: String,
        multipart: multipart::Form,
        headers: Option<HeaderMap>,
    ) -> Result<String, Box<dyn Error>> {
        let all_headers = self.merge_headers(headers);
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

    /// Get multipart Part .zip archive
    fn get_part_zip_archive(
        &self,
        full_file_path_to_archive: &PathBuf,
    ) -> Result<multipart::Part, Box<dyn Error>> {
        // Читаем массив байтов
        let file_to_bytes = std::fs::read(full_file_path_to_archive).map_err(|e| {
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

    // pub async fn get_launch_by_id(
    //     &self,
    //     launch_id: u32,
    // ) -> Result<GetLauncByIdResponce, Box<dyn Error>> {
    //     let response = self.get(format!("/launch/{}", launch_id), None).await?;
    //     if response.is_empty() {
    //         return Err(WotApiError::EmptyResponse("/launch/[launch_id]".to_string()).into());
    //     }
    //     match serde_json::from_str(&response) {
    //         Ok(value) => Ok(value),
    //         Err(e) => Err(WotApiError::ParsingResponse(
    //             "/launch/[launch_id]".to_string(),
    //             e.to_string(),
    //         )
    //         .into()),
    //     }
    // }

    /// Upload zip archiver report to launch TestOps
    ///
    /// Param:
    ///
    /// full_file_path_to_archive: full path to archive file with test results
    /// launch_info: struct LaunchInfo { name, project_id }
    pub async fn post_archive_report_launch_upload(
        &self,
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

    /// Get all project_ids
    pub async fn get_all_project_ids(&self) -> Result<HashSet<u32>, Box<dyn Error>> {
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

    /// Get project info by project_id
    pub async fn get_project_info_by_id(
        &self,
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

    /// Get testcasse overview by testcase_id
    pub async fn get_test_case_overview_by_id(
        &self,
        test_case_id: u32,
    ) -> Result<TestCaseOverview, Box<dyn Error>> {
        let response = self
            .get(format!("/testcase/{}/overview", test_case_id), None)
            .await?;
        if response.is_empty() {
            return Err(WotApiError::EmptyResponse("/testcase/<id>/overview".to_string()).into());
        }
        match serde_json::from_str::<TestCaseOverview>(&response) {
            Ok(value) => Ok(value),
            Err(e) => Err(WotApiError::ParsingResponse(
                "/testcase/<id>/overview".to_string(),
                e.to_string(),
            )
            .into()),
        }
    }

    pub async fn get_testcase_scenario(&self, test_case_id: u32) -> Result<Scenario, Box<dyn Error>> {
        let response = self.get(format!("/testcase/{}/step", test_case_id), None).await?;
        if response.is_empty() {
            return Err(WotApiError::EmptyResponse("/testcase/<id>/step".to_string()).into());
        }
        match serde_json::from_str::<Scenario>(&response) {
            Ok(value) => Ok(value),
            Err(e) => Err(WotApiError::ParsingResponse(
                "/testcase/<id>/step".to_string(),
                e.to_string(),
            )
            .into()),
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

#[derive(Deserialize, Serialize, Debug)]
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

enum AllureMetaData {
    Epic,
    Feature,
    Story,
    Suite,
    Unknown,
}

impl From<String> for AllureMetaData {
    fn from(value: String) -> Self {
        match value.as_str() {
            "Feature" => AllureMetaData::Feature,
            "Epic" => AllureMetaData::Epic,
            "Story" => AllureMetaData::Story,
            "Suite" => AllureMetaData::Suite,
            _ => AllureMetaData::Unknown,
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TestCaseOverview {
    id: u32,
    project_id: u32,
    name: String,
    description: Option<String>,
    precondition: Option<String>,
    expected_result: Option<String>,
    custom_fields: Option<Vec<CustomFieldInfo>>,
    tags: Option<Vec<Tag>>,
}

impl TestCaseOverview {
    /// Convert allure metadata
    fn convert_allure_metadata_to_python_template(&self) -> String {
        let mut vec_string: Vec<String> = vec![];
        let custom_fields: Vec<CustomFieldInfo> = match self.custom_fields.clone() {
            Some(value) => value,
            None => return String::from(""),
        };
        for i in custom_fields {
            let meta_data_type = AllureMetaData::from(i.custom_field.name.clone());
            match meta_data_type {
                AllureMetaData::Epic => vec_string.push(format!("@allure.epic('{}')", i.name)),
                AllureMetaData::Feature => {
                    vec_string.push(format!("@allure.feature('{}')", i.name))
                }
                AllureMetaData::Story => vec_string.push(format!("@allure.story('{}')", i.name)),
                AllureMetaData::Suite => vec_string.push(format!("@allure.suite('{}')", i.name)),
                _ => {
                    vec_string.push(format!("@allure.label('{}', '{}')", i.custom_field.name.to_lowercase(), i.name));
                },
            }
        }
        let allure_tags: String = self.tags.as_ref().map_or_else(
            || String::from(""),
            |value| {
                value.iter().map(|i| format!("'{}'", i.name)).collect::<Vec<String>>().join(", ")
            });
        vec_string.push(format!("@allure.tag({})", allure_tags));
        vec_string.join("\n")
    }

    /// Collect docstring for testcase
    fn concat_all_description(&self) -> String {
        let mut all_description: Vec<String> = vec![];
        if let Some(description) = self.description.clone() {
            all_description.push(description)
        }
        if let Some(prediction) = self.precondition.clone() {
            all_description.push(prediction);
        }
        if let Some(exp_res) = self.expected_result.clone() {
            all_description.push(exp_res)
        }
        all_description.join("\n\n").replace("\n", "\n\t\t")
    }

}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CustomFieldInfo {
    id: u32,
    name: String,
    custom_field: CustomField,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CustomField {
    name: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Step {
    name: String,
    steps: Vec<Self>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Tag {
    id: u32,
    name: String
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Scenario {
    root: Root,
    pub scenario_steps: HashMap<String, ScenarioStep>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Root {
    children: Vec<u64>
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ScenarioStep {
    id: u64,
    body: String,
    #[serde(default)]
    expected_result_id: Option<u64>,
    #[serde(default)]
    children: Option<Vec<u64>>,
}

impl Scenario {
    fn get_scenario(&self) -> String {
        let mut step_strings: Vec<String> = vec![];
        for &id in &self.root.children {
            let step_key = id.to_string();
            if let Some(step) = self.scenario_steps.get(&step_key) {
                step_strings.push(step.body.clone());
                step.expected_result_id
                    .and_then(|eid| self.scenario_steps.get(&eid.to_string()))
                    .filter(|estep| estep.body == "Expected Result")
                    .and_then(|estep| estep.children.as_ref())
                    .into_iter()
                    .flatten()
                    .filter_map(|&cid| self.scenario_steps.get(&cid.to_string()))
                    .map(|child_step| format!("\t{}", child_step.body))
                    .for_each(|step| step_strings.push(step));
            }
        }
        step_strings.join("\n\t\t\t")
    }
}

pub fn create_template_python_ati_su(test_case_overview: TestCaseOverview,
    test_case_scenario: Scenario, file_name: &str) -> Result<String, WotError>{
    let allure_metadata = test_case_overview.convert_allure_metadata_to_python_template();
    let all_description = test_case_overview.concat_all_description();
    let scenario = test_case_scenario.get_scenario();
    let template = format!(
        "import pytest
import allure


{}
@pytest.mark.TEMPLATE_MARK_NAME
class Test1:

    @allure.id('{}')
    @allure.title('{}')
    def test1(self):
        \"\"\"
        {}

        {}

        Шаги:
            {}
        \"\"\"
        pass
",
        allure_metadata,
        test_case_overview.id,
        test_case_overview.name,
        test_case_overview.name,
        all_description,
        scenario,
    );
    create_file_in_current_directory(file_name, template.as_bytes())
}

#[cfg(test)]
mod tests {

    use super::*;
    use std::env;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

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

    // #[tokio::test]
    // async fn test_get_all_project_ids() {
    //     let resp = testops_api_client().get_all_project_ids().await.unwrap();
    //     assert!(resp.contains(&2), "Не нашли проект с id == 2");
    // }

    // #[tokio::test]
    // async fn test_get_project_by_id() {
    //     let resp = testops_api_client()
    //         .get_project_info_by_id(&2)
    //         .await
    //         .unwrap();
    //     assert_eq!("TestProject", resp.name);
    // }

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
        let exp_err = "Need a file with a .zip extension, a file was transferred: *.json";
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
        let exp_err = "Couldn't read the file in the path: \
            \"/Users/valentins/Desktop/rust_projects/wot/test_files/\". \
            We got an error: Is a directory (os error 21)"
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
        let exp_err = "Couldn't read the file in the path: \"\". \
            We got an error: No such file or directory (os error 2)"
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
    #[test]
    fn test_scenario_parse() {
        let data = fs::read_to_string(format!("{}/test_files/scenario_with_expected_result.json", env!("CARGO_MANIFEST_DIR"))).unwrap();
        let scenario: Scenario = serde_json::from_str(&data).expect("Ошибка парсинга JSON из файла /test_files/scenario_with_expected_result.json");
        let exp_str = "Подготовка к тесту
\t\t\t\tПроверка после подготовки
\t\t\tВторой шаг, что то дергаем
\t\t\tЗавершаем тест
\t\t\tДобавили еще один шаг
\t\t\tПервый шаг, создаем юзера
\t\t\t\tПроверяем 200 и user_id не пустой
\t\t\t\tи что нибудь еще
\t\t\t\tИ тут нужна еще одна проверка";
        assert_eq!(scenario.get_scenario(), exp_str);
    }

    #[test]
    fn test_empty_scenario_parse() {
        let json_str = r#"
        {
            "root": {
            "children": []
            },
            "scenarioSteps": {},
            "attachments": {},
            "sharedSteps": {},
            "sharedStepScenarioSteps": {},
            "sharedStepAttachments": {}
        }
        "#;
        let data: Scenario = serde_json::from_str(json_str).unwrap();
        let exp_str = "";
        assert_eq!(data.get_scenario(), exp_str);
    }

    #[test]
    fn test_create_template_overview_and_scenario() {
        let data_scenario = fs::read_to_string(format!("{}/test_files/scenario_with_expected_result.json", env!("CARGO_MANIFEST_DIR"))).unwrap();
        let scenario: Scenario = serde_json::from_str(&data_scenario).expect("Ошибка парсинга JSON из файла /test_files/scenario_with_expected_result.json");
        let data_overview = fs::read_to_string(format!("{}/test_files/test_case_overview_24442.json", env!("CARGO_MANIFEST_DIR"))).unwrap();
        let overview: TestCaseOverview = serde_json::from_str(&data_overview).expect("Ошибка парсинга JSON из файла /test_files/test_case_overview_24442.json");
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let file_name = format!("test_{}_{}.py", timestamp, 24442);
        let _ = create_template_python_ati_su(overview, scenario, &file_name);
    }
}
