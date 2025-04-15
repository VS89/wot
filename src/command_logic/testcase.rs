use crate::create_template::ati_su_python_template_test::create_template_python_ati_su;
use crate::external_api::testops_api::TestopsApi;
use crate::external_api::ApiError;
use std::time::{SystemTime, UNIX_EPOCH};

/// Import testcase by id from TestOps
pub async fn import_testcase_by_id(
    test_case_id: u32,
    testops_api_client: &TestopsApi,
) -> Result<String, ApiError> {
    let test_case_overview = testops_api_client
        .get_test_case_overview_by_id(&test_case_id)
        .await
        .map_err(|_| ApiError::CouldNotFindTestCaseById(test_case_id))?;
    let test_case_scenario = testops_api_client
        .get_test_case_scenario(&test_case_id)
        .await
        .map_err(|_| ApiError::CouldNotFindTestCaseById(test_case_id))?;
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let file_name = format!("test_{}_{}.py", timestamp, test_case_id);
    let full_path_to_file =
        create_template_python_ati_su(test_case_overview, test_case_scenario, &file_name).await?;
    Ok(format!("File created: {}", full_path_to_file))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::external_api::ApiError;
    use mockito::Server;

    #[tokio::test]
    async fn test_import_testcase_by_id_not_found() {
        let mut server = Server::new_async().await;
        let test_case_id = 999;

        let mock = server
            .mock(
                "GET",
                format!("/api/rs/testcase/{}/overview", test_case_id).as_str(),
            )
            .with_status(404)
            .create_async()
            .await;

        let api_client = TestopsApi::mock(&server.url());

        let result = import_testcase_by_id(test_case_id, &api_client).await;

        assert!(matches!(
            result,
            Err(ApiError::CouldNotFindTestCaseById(999))
        ));
        mock.assert();
    }

    #[tokio::test]
    async fn test_import_testcase_by_id_server_error() {
        let mut server = Server::new_async().await;
        let test_case_id = 123;

        let mock = server
            .mock(
                "GET",
                format!("/api/rs/testcase/{}/overview", test_case_id).as_str(),
            )
            .with_status(500)
            .create_async()
            .await;

        let api_client = TestopsApi::mock(&server.url());

        let result = import_testcase_by_id(test_case_id, &api_client).await;

        assert!(matches!(
            result,
            Err(ApiError::CouldNotFindTestCaseById(123))
        ));
        mock.assert();
    }
}
