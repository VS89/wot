pub mod models;
pub mod allure_meta_data;

use std::path::Path;
use std::collections::HashSet;
use reqwest::multipart::{Form, Part};
use super::{BaseApiClient, ApiError};
use crate::utils::{read_file_to_buffer, get_file_name, validate_zip_archive};
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
            project_ids.extend(
                response
                    .content
                    .iter()
                    .map(|project_info| project_info.id),
            );
            if response.total_pages <= (current_page + 1) || current_page >= limit_pages
            {
                break;
            };
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

    pub async fn get_test_case_scenario(&self, test_case_id: &u32) -> Result<Scenario, ApiError> {
        self.client.get::<Scenario, ()>(&format!("{}/testcase/{}/step", self.api_prefix, test_case_id)).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Server;
    use crate::constants::CARGO_MANIFEST_DIR;

    impl TestopsApi {

        pub fn default_test() -> Self {
            use std::env;
            let base_url = env::var("TESTOPS_BASE_URL").unwrap();
            let api_key = env::var("TESTOPS_API_TOKEN").unwrap();
            let base_api_client = BaseApiClient::new(&base_url, &api_key).unwrap();
            Self { client: base_api_client, api_prefix: "/api/rs".to_string() }
        }
    
        pub fn mock(base_url: &str) -> Self {
            use std::env;
            let api_key = env::var("TESTOPS_API_TOKEN").unwrap();
            let base_api_client = BaseApiClient::new(&base_url, &api_key).unwrap();
            Self { client: base_api_client, api_prefix: "/api/rs".to_string() }
        }
    
        pub async fn mock_post_upload_report(
            server_mock: &mut mockito::ServerGuard, 
            mock_response: &ResponseLaunchUpload
        ) {
            server_mock.mock("POST", "/api/rs/launch/upload")
                .match_header("content-type", mockito::Matcher::Regex(r"multipart/form-data; boundary=.*".into()))
                .match_body(mockito::Matcher::Regex("Content-Disposition: form-data; name=\"info\"".into()))
                .match_body(mockito::Matcher::Regex("Content-Disposition: form-data; name=\"archive\"".into()))
                .with_status(200)
                .with_header("content-type", "application/json")
                .with_body(serde_json::to_string(&mock_response).unwrap())
                .create_async().await;
        }

        pub async fn mock_get_project_by_id(server_mock: &mut mockito::ServerGuard, mock_response: &ProjectInfo) {
            let endpoint = mockito::Matcher::Exact(format!("/api/rs/project/{}", mock_response.id));
            server_mock.mock("GET", endpoint)
                .with_status(200)
                .with_header("content-type", "application/json")
                .with_body(serde_json::to_string(&mock_response).unwrap())
                .create_async().await;
        }

        pub async fn mock_get_launch_by_id(
            server_mock: &mut mockito::ServerGuard, 
            mock_response: &GetLaunchByIdResponse, 
            launch_id: u32
        ) {
            let endpoint = mockito::Matcher::Exact(format!("/api/rs/launch/{}", launch_id));
            server_mock.mock("GET", endpoint)
                .with_status(200)
                .with_header("content-type", "application/json")
                .with_body(serde_json::to_string(&mock_response).unwrap())
                .create_async().await;
        }

        pub async fn mock_get_all_projects(
            server_mock: &mut mockito::ServerGuard,
        ) {
            let response_page_0 = ResponseGetAllProject { 
                total_pages: 2, 
                content: vec![
                    ProjectInfo::new(1, "Project1"),
                    ProjectInfo::new(2, "Test Project"),
                ],
            };
            let response_page_1 = ResponseGetAllProject {
                total_pages: 2, 
                content: vec![
                    ProjectInfo::new(3, "Project3"),
                ],  
            };
            server_mock.mock("GET", "/api/rs/project?page=0")
                .with_status(200)
                .with_header("content-type", "application/json")
                .with_body(serde_json::to_string(&response_page_0).unwrap())
                .create_async().await;

            server_mock.mock("GET", "/api/rs/project?page=1")
                .with_status(200)
                .with_header("content-type", "application/json")
                .with_body(serde_json::to_string(&response_page_1).unwrap())
                .create_async().await;
        }

        pub async fn mock_get_test_case_overview_by_id(
            server_mock: &mut mockito::ServerGuard,
            mock_response: &TestCaseOverview,
        ) {
            let endpoint = mockito::Matcher::Exact(format!("/api/rs/testcase/{}/overview", &mock_response.id));
            server_mock.mock("GET", endpoint)
                .with_status(200)
                .with_header("content-type", "application/json")
                .with_body(serde_json::to_string(&mock_response).unwrap())
                .create_async().await;
        }

        pub async fn mock_get_test_case_scenario(
            server_mock: &mut mockito::ServerGuard,
            mock_response: &Scenario,
            test_case_id: u32,
        ) {
            let endpoint = mockito::Matcher::Exact(format!("/api/rs/testcase/{}/step", test_case_id));
            server_mock.mock("GET", endpoint)
                .with_status(200)
                .with_header("content-type", "application/json")
                .with_body(serde_json::to_string(&mock_response).unwrap())
                .create_async().await;
        }
    
    }

    #[tokio::test]
    async fn test_post_upload_report_mock() {
        let exp_launch_id = 11111;
        // Мокаем данные
        let mut server = Server::new_async().await;
        let mock_response = ResponseLaunchUpload::default();
        let testops_api = TestopsApi::mock(&server.url());
        TestopsApi::mock_post_upload_report(&mut server, &mock_response).await;
        let path_to_report = Path::new(CARGO_MANIFEST_DIR).join("test_files/test_upload_launch_report.zip");
        // Проверяем ответ от метода
        let result = testops_api.post_upload_report(&path_to_report, &LaunchInfo::default()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().launch_id, exp_launch_id);
    }

    #[tokio::test]
    async fn test_get_project_info_by_id() {
        let project_id = 2;
        let project_name = "Test Project";
        let mock_response = ProjectInfo::new(project_id, project_name);
        let mut server = Server::new_async().await;
        let testops_api = TestopsApi::mock(&server.url());
        TestopsApi::mock_get_project_by_id(&mut server, &mock_response).await;

        let result = testops_api.get_project_info_by_id(&project_id).await;
        assert!(result.is_ok());
        let unwrap_result = result.unwrap();
        assert_eq!(unwrap_result.id, project_id);
        assert_eq!(unwrap_result.name, project_name);
    }

    #[tokio::test]
    async fn test_get_launch_by_id() {
        let launch_id = 22222;
        let mock_response = GetLaunchByIdResponse::new(launch_id, "MyLaunchName", 2);
        let mut server = Server::new_async().await;
        let testops_api = TestopsApi::mock(&server.url());
        TestopsApi::mock_get_launch_by_id(&mut server, &mock_response, launch_id).await;

        let result = testops_api.get_launch_by_id(launch_id).await;
        assert!(result.is_ok());
        let unwrap_result = result.unwrap();
        assert_eq!(unwrap_result.id, launch_id);

    }

    #[tokio::test]
    async fn test_get_all_project_ids() {
        let exp_len_elements = 3;
        let mut exp_response_hash_set = HashSet::with_capacity(3);
        exp_response_hash_set.insert(1);
        exp_response_hash_set.insert(2);
        exp_response_hash_set.insert(3);
        let mut server = Server::new_async().await;
        let testops_api = TestopsApi::mock(&server.url());
        TestopsApi::mock_get_all_projects(&mut server).await;

        let result = testops_api.get_all_project_ids().await;
        assert!(result.is_ok());
        let unwrap_result = result.unwrap();
        assert_eq!(exp_len_elements, unwrap_result.len());
        assert_eq!(exp_response_hash_set, unwrap_result);
    }

    #[test]
    fn test_field_api_prefix() {
        let testops_api = TestopsApi::default_test();
        assert_eq!(testops_api.api_prefix, "/api/rs")
    }

    #[tokio::test]
    async fn test_get_project_info_by_id_nonexistent_project() {
        let testops_api = TestopsApi::default_test();
        let response = testops_api.get_project_info_by_id(&9999).await;
        assert!(response.is_err());
        assert!(matches!(response.unwrap_err(), ApiError::Api(_, _)));
    }

    #[tokio::test]
    async fn test_get_test_case_overview() {
        let testops_api = TestopsApi::default_test();
        let response = testops_api.get_test_case_overview_by_id(&24013).await;
        assert!(response.is_ok());
        assert_eq!(response.unwrap().id, 24013);
    }

    #[tokio::test]
    async fn test_get_test_case_overview_nonexistent_test() {
        let testops_api = TestopsApi::default_test();
        let response = testops_api.get_test_case_overview_by_id(&99999).await;
        assert!(response.is_err());
        assert!(matches!(response.unwrap_err(), ApiError::Api(_, _)));
    }
}