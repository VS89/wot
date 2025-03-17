use crate::config::Config;
use crate::errors::WotError;
use crate::external_api::testops::{TestopsApiClient, create_template_python_ati_su};
use std::time::{SystemTime, UNIX_EPOCH};

/// Import testcase by id from TestOps
pub async fn import_testcase_by_id(
    test_case_id: u32,
    config: &Config,
) -> Result<(), WotError> {
    let testops = TestopsApiClient::new(config);
    let test_case_overview = match testops
        .get_test_case_overview_by_id(test_case_id)
        .await {
            Ok(value) => value,
            Err(_) => {
                let error_text = WotError::CouldNotFindTestCaseById(test_case_id.to_string());
                return Err(error_text);
            }
        };
    let test_case_scenario = match testops.get_testcase_scenario(test_case_id).await {
        Ok(value) => value,
        Err(_) => {
            let error_text = WotError::CouldNotFindTestCaseById(test_case_id.to_string());
            return Err(error_text);
        }
    };
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let file_name = format!("test_{}_{}.py", timestamp, test_case_id);
    let full_path_to_file = create_template_python_ati_su(test_case_overview, test_case_scenario, &file_name)?;
    println!("File created: {}", full_path_to_file);
    Ok(())
}



#[cfg(test)]
mod tests {

    use super::*;
    use std::env;

    #[tokio::test]
    async fn test_get_test_case_overview_negative() {
        let test_case_id = 23222292;
        let config = Config {
            testops_base_url: env::var("TESTOPS_BASE_URL").unwrap(),
            testops_api_token: env::var("TESTOPS_API_TOKEN").unwrap(),
        };
        let resp = import_testcase_by_id(test_case_id, &config).await;
        assert_eq!(format!("Couldn't find a test case with ID == {}", test_case_id), resp.unwrap_err().to_string());
    }
}
