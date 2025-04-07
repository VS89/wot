use crate::ApiError;
use crate::external_api::testops_api::models::test_case_overview::TestCaseOverview;
use crate::external_api::testops_api::models::test_case_scenario::Scenario;
use crate::utils::create_file_in_current_directory;

pub async fn create_template_python_ati_su(test_case_overview: TestCaseOverview,
    test_case_scenario: Scenario, file_name: &str) -> Result<String, ApiError>{
    let allure_metadata = test_case_overview.convert_allure_metadata_to_python_template();
    let all_description = test_case_overview.concat_all_description();
    let scenario = test_case_scenario.get_scenario();
    let template = format!(
        r#"
import pytest
import allure
{}
{}
@pytest.mark.TEMPLATE_MARK_NAME
class Test1:

    @allure.id('{}')
    @allure.title('{}')
    def test1(self):
        """
        {}

        {}

        Шаги:
            {}
        """
        pass
"#,
        if allure_metadata.is_empty() {""} else { "\n" },
        allure_metadata,
        test_case_overview.id,
        test_case_overview.name,
        test_case_overview.name,
        all_description,
        scenario,
    );
    create_file_in_current_directory(file_name, template.as_bytes()).await
}

// #[cfg(test)]
// mod tests {
//     use std::collections::HashMap;

//     use super::*;
// }