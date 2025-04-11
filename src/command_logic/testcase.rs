use crate::external_api::ApiError;
use crate::external_api::testops_api::TestopsApi;
use crate::create_template::ati_su_python_template_test::create_template_python_ati_su;
use std::time::{SystemTime, UNIX_EPOCH};

/// Import testcase by id from TestOps
pub async fn import_testcase_by_id(
    test_case_id: u32,
    testops_api_client: &TestopsApi,
) -> Result<(), ApiError> {
    let test_case_overview = testops_api_client.get_test_case_overview_by_id(&test_case_id)
        .await.map_err(|_| ApiError::CouldNotFindTestCaseById(test_case_id))?;
    let test_case_scenario = testops_api_client.get_test_case_scenario(&test_case_id)
        .await.map_err(|_| ApiError::CouldNotFindTestCaseById(test_case_id))?;
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let file_name = format!("test_{}_{}.py", timestamp, test_case_id);
    let full_path_to_file = create_template_python_ati_su(test_case_overview, test_case_scenario, &file_name).await?;
    println!("File created: {}", full_path_to_file);
    Ok(())
}


// todo нужны кейсы 
// #[cfg(test)]
// mod tests {

//     use super::*;
//     use std::env;

//     #[tokio::test]
//     async fn test_get_test_case_overview_negative() {
//         let test_case_id = 23222292;
//         let config = Config {
//             testops_base_url: env::var("TESTOPS_BASE_URL").unwrap(),
//             testops_api_token: env::var("TESTOPS_API_TOKEN").unwrap(),
//         };
//         let resp = import_testcase_by_id(test_case_id, &config).await;
//         assert_eq!(format!("Couldn't find a test case with ID == {}", test_case_id), resp.unwrap_err().to_string());
//     }
// }
