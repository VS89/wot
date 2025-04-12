use crate::ApiError;
use crate::external_api::testops_api::models::test_case_overview::TestCaseOverview;
use crate::external_api::testops_api::models::test_case_scenario::Scenario;
use crate::utils::save_file_in_current_directory;

fn generate_python_template(
    test_case_overview: &TestCaseOverview,
    test_case_scenario: &Scenario,
) -> String {
    let allure_metadata = test_case_overview.convert_allure_metadata_to_python_template();
    let prefix_newline = if allure_metadata.is_empty() { "" } else { "\n" };
    let all_description = test_case_overview.concat_all_description();
    let scenario = test_case_scenario.get_scenario();

    format!(
        r#"
import pytest
import allure
{prefix_newline}{allure_metadata}
@pytest.mark.TEMPLATE_MARK_NAME
class Test1:

    @allure.id('{allure_id}')
    @allure.title('{name}')
    def test1(self):
        """
        {name}

        {all_description}

        Шаги:
            {scenario}
        """
        pass
"#,
        prefix_newline = prefix_newline,
        allure_metadata = allure_metadata,
        allure_id = test_case_overview.id,
        name = &test_case_overview.name,
        all_description = all_description,
        scenario = scenario,
    )
}


pub async fn create_template_python_ati_su(test_case_overview: TestCaseOverview,
    test_case_scenario: Scenario, file_name: &str) -> Result<String, ApiError>{
    let template = generate_python_template(&test_case_overview, &test_case_scenario);
    save_file_in_current_directory(file_name, template.as_bytes()).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::external_api::testops_api::models::test_case_overview::TestCaseOverview;
    use crate::external_api::testops_api::models::test_case_scenario::Scenario;

    #[test]
    fn test_generate_template_minimum() {
        let test_case_overview = TestCaseOverview::default();
        let scenario = Scenario::default();
        let template = generate_python_template(&test_case_overview, &scenario);
        let exp_template = r#"
import pytest
import allure

@pytest.mark.TEMPLATE_MARK_NAME
class Test1:

    @allure.id('1234')
    @allure.title('Some name case')
    def test1(self):
        """
        Some name case

        

        Шаги:
            111_body
        """
        pass
"#;
        assert_eq!(exp_template, template);
    }
}