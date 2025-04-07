pub mod models;
pub mod allure_meta_data;

use std::path::Path;
use std::collections::HashSet;
use reqwest::multipart::{Form, Part};
use super::{BaseApiClient, ApiError};
use crate::utils::{read_file_to_buffer, get_file_name, validate_zip_archive};
use models::project::Project;
use models::get_launch_by_id::GetLaunchByIdResponse;
use models::response_launch_upload::ResponseLaunchUpload;
use models::launch_info::LaunchInfo; 
use models::response_get_all_project::ResponseGetAllProject; 
use models::project_info::ProjectInfo;  
use models::test_case_overview::TestCaseOverview;
use models::test_case_scenario::Scenario;   


pub struct TestopsApi {
    pub client: BaseApiClient,
    api_prefix: String,
}

impl TestopsApi {
    pub fn new(api_key: &str, base_url: &str) -> Self {
        let base_api_client = BaseApiClient::new(base_url, api_key).unwrap();
        Self { client: base_api_client, api_prefix: "/api/rs".to_string() }
    }

    #[cfg(test)]
    pub fn default() -> Self {
        use std::env;
        let base_url = env::var("TESTOPS_BASE_URL").unwrap();
        let api_key = env::var("TESTOPS_API_TOKEN").unwrap();
        let base_api_client = BaseApiClient::new(&base_url, &api_key).unwrap();
        Self { client: base_api_client, api_prefix: "/api/rs".to_string() }
    }

    pub async fn get_project_by_id(&self, id: u8) -> Result<Project, ApiError> {
        self.client.get::<Project, ()>(&format!("{}/project/{}", self.api_prefix, id)).await
    }

    pub async fn get_launch_by_id(&self, launch_id: u32) -> Result<GetLaunchByIdResponse, ApiError> {
        self.client.get::<GetLaunchByIdResponse, ()>(&format!("{}/launch/{}", self.api_prefix, launch_id)).await
    }

    pub async fn post_upload_report(&self, file_path: &Path, launch_info: &LaunchInfo) -> Result<ResponseLaunchUpload, ApiError> {
        // Читаем в буфер файл
        let buffer = read_file_to_buffer(file_path).await?;

        // Создаем multipart с данными файла
        let file_name = get_file_name(file_path)?;
        validate_zip_archive(&buffer)?;
        let file_part = Part::bytes(buffer).file_name(file_name).mime_str("application/zip").unwrap();
        let info_file_json = serde_json::to_string(launch_info)?;
        let info_file_multipart = Part::text(info_file_json).mime_str("application/json").unwrap();

        // Собираем форму с файлом
        let form = Form::new().part("info", info_file_multipart).part("archive", file_part);
        self.client.post_multipart_file::<ResponseLaunchUpload, ()>(&format!("{}/launch/upload", self.api_prefix), form).await
    }

    pub async fn get_all_project_ids(&self) -> Result<HashSet<u32>, ApiError> {    
        let mut current_page: u32 = 0;
        let limit_pages: u32 = 50;
        let mut project_ids: HashSet<u32> = HashSet::new();
        loop {
            let response = self.client.get::<ResponseGetAllProject, ()>(
                &format!("{}/project?page={}", self.api_prefix, current_page)).await?;
            if response.total_pages < (current_page + 1) || current_page >= limit_pages
            {
                break;
            };
            project_ids.extend(
                response
                    .content
                    .iter()
                    .map(|project_info| project_info.id),
            );
            current_page += 1;
        }
        Ok(project_ids)
    }

    pub async fn get_project_info_by_id(&self, project_id: &u32) -> Result<ProjectInfo, ApiError> {
        self.client.get::<ProjectInfo, ()>(&format!("{}/project/{}", self.api_prefix, project_id)).await
    }

    pub async fn get_test_case_overview_by_id(&self, test_case_id: &u32) -> Result<TestCaseOverview, ApiError> {
        self.client.get::<TestCaseOverview, ()>(&format!("{}/testcase/{}/overview", self.api_prefix, test_case_id)).await
    }

    pub async fn get_testcase_scenario(&self, test_case_id: &u32) -> Result<Scenario, ApiError> {
        self.client.get::<Scenario, ()>(&format!("{}/testcase/{}/step", self.api_prefix, test_case_id)).await
    }
}

#[cfg(test)]
mod tests {
    use std::env;
    use super::*;

    const TEST_PROJECT_ID: &u32 = &2;
    const TEST_PROJECT_NAME: &str = "TestProject";

    #[test]
    fn test_field_api_prefix() {
        let testops_api = TestopsApi::default();
        assert_eq!(testops_api.api_prefix, "/api/rs")
    }

    #[tokio::test]
    async fn test_get_projet_by_id() {
        // Создаем структуру
        let testops_api = TestopsApi::default();
        let resp: Project = testops_api.get_project_by_id(2).await.unwrap();
        assert_eq!("TestProject", resp.name);
        assert_eq!(2, resp.id);
    }

    #[tokio::test]
    async fn test_upload_report() {
        let file_path = Path::new(env::var("CARGO_MANIFEST_DIR").unwrap().as_str())
            .join("test_files/testops_results_report_1735389182.zip");
        let launch_info = LaunchInfo::default();
        let testops_api = TestopsApi::default();
        let resp: ResponseLaunchUpload = testops_api.post_upload_report(&file_path, &launch_info).await.unwrap();
        assert!(resp.launch_id != 0);
    }

    #[tokio::test]
    async fn test_get_all_project_ids() {
        let testops_api = TestopsApi::default();
        let response = testops_api.get_all_project_ids().await;
        assert!(response.is_ok());
        assert!(response.unwrap().contains(TEST_PROJECT_ID), "Не нашли проект с id == {}", TEST_PROJECT_ID);
    }

    #[tokio::test]
    async fn test_get_project_info_by_id() {
        let testops_api = TestopsApi::default();
        let response = testops_api.get_project_info_by_id(TEST_PROJECT_ID).await;
        assert!(response.is_ok());
        assert_eq!(TEST_PROJECT_NAME, response.unwrap().name);
    }

    #[tokio::test]
    async fn test_get_project_info_by_id_nonexistent_project() {
        let testops_api = TestopsApi::default();
        let response = testops_api.get_project_info_by_id(&9999).await;
        assert!(response.is_err());
        assert!(matches!(response.unwrap_err(), ApiError::Api(_, _)));
    }

    #[tokio::test]
    async fn test_get_test_case_overview() {
        let testops_api = TestopsApi::default();
        let response = testops_api.get_test_case_overview_by_id(&24013).await;
        assert!(response.is_ok());
        assert_eq!(response.unwrap().id, 24013);
    }

    #[tokio::test]
    async fn test_get_test_case_overview_nonexistent_test() {
        let testops_api = TestopsApi::default();
        let response = testops_api.get_test_case_overview_by_id(&99999).await;
        assert!(response.is_err());
        assert!(matches!(response.unwrap_err(), ApiError::Api(_, _)));
    }
}