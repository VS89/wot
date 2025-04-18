use crate::external_api::testops_api::models::test_case_overview::TestCaseOverview;
use crate::external_api::testops_api::models::test_case_scenario::Scenario;
use crate::utils::{convert_to_pascal_case, save_file_in_current_directory};
use crate::ApiError;

fn generate_python_template(
    test_case_overview: &TestCaseOverview,
    test_case_scenario: &Scenario,
    file_name: &str,
) -> String {
    let allure_metadata = test_case_overview.convert_allure_metadata_to_python_template();
    let prefix_newline = if allure_metadata.is_empty() { "" } else { "\n" };
    let all_description = test_case_overview.concat_all_description();
    let scenario = test_case_scenario.get_scenario();
    let file_name_without_extension = file_name.strip_suffix(".py").unwrap_or_default();
    let class_name = convert_to_pascal_case(file_name_without_extension);

    format!(
        r#"
import pytest
import allure
{prefix_newline}{allure_metadata}
@pytest.mark.TEMPLATE_MARK_NAME
class {class_name}:

    @allure.id('{allure_id}')
    @allure.title('{name}')
    def {file_name_without_extension}(self):
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
        class_name = class_name,
        allure_id = test_case_overview.id,
        name = &test_case_overview.name,
        file_name_without_extension = file_name_without_extension,
        all_description = all_description,
        scenario = scenario,
    )
}

pub async fn create_template_python_ati_su(
    test_case_overview: TestCaseOverview,
    test_case_scenario: Scenario,
    file_name: &str,
) -> Result<String, ApiError> {
    let template = generate_python_template(&test_case_overview, &test_case_scenario, file_name);
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
        let file_name = "test_case_one.py";
        let template = generate_python_template(&test_case_overview, &scenario, file_name);
        let exp_template = r#"
import pytest
import allure

@pytest.mark.TEMPLATE_MARK_NAME
class TestCaseOne:

    @allure.id('1234')
    @allure.title('Some name case')
    def test_case_one(self):
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
