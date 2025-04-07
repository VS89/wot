use crate::external_api::testops_api::allure_meta_data::AllureMetaData;
use super::custom_field_info::CustomFieldInfo;
use super::tag::Tag;

#[derive(serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TestCaseOverview {
    pub id: u32,
    pub project_id: u32,
    pub name: String,
    pub description: Option<String>,
    pub precondition: Option<String>,
    pub expected_result: Option<String>,
    pub custom_fields: Option<Vec<CustomFieldInfo>>,
    pub tags: Option<Vec<Tag>>,
}

impl TestCaseOverview {

    #[cfg(test)]
    pub fn default() -> Self {
        Self { 
            id: 1234, 
            project_id: 222, 
            name: "Some name case".to_string(), 
            description: None, 
            precondition: None, 
            expected_result: None, 
            custom_fields: None, 
            tags: None 
        }
    }
}

impl TestCaseOverview {

    fn generate_allure_decorators_from_fields(&self) -> Vec<String> {
        self.custom_fields.as_ref().map_or_else(Vec::new, |fields| {
            fields.iter()
                .map(|field| {
                    match field.custom_field.name.to_ascii_lowercase().as_str() {
                        "epic" => AllureMetaData::epic(&field.name),
                        "feature" => AllureMetaData::feature(&field.name),
                        "story" => AllureMetaData::story(&field.name),
                        "suite" => AllureMetaData::suite(&field.name),
                        _ => AllureMetaData::label(&field.custom_field.name, &field.name),
                    }
                })
                .collect()
        })
    }

    /// Convert allure metadata
    pub fn convert_allure_metadata_to_python_template(&self) -> String {
        let mut allure_decorators = self.generate_allure_decorators_from_fields();
        if let Some(tags) = &self.tags {
            let tag_list = tags.iter()
                .map(|t| format!("'{}'", t.name))
                .collect::<Vec<_>>()
                .join(", ");
            if !tag_list.is_empty() {
                allure_decorators.push(format!("@allure.tag({})", tag_list));
            }
        }
        allure_decorators.join("\n")
    }

    /// Collect docstring for testcase
    pub fn concat_all_description(&self) -> String {
        [self.description.as_deref(), self.precondition.as_deref(), self.expected_result.as_deref()]
            .iter()
            .filter_map(|&part| part)
            .collect::<Vec<&str>>()
            .join("\n\n")
    }

}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::external_api::testops_api::models::custom_field::CustomField;
    use crate::external_api::testops_api::models::custom_field_info::CustomFieldInfo;

    fn create_custom_field(name: &str, value: &str) -> CustomFieldInfo {
        CustomFieldInfo {
            id: 1,
            custom_field: CustomField { name: name.to_string() },
            name: value.to_string(),
        }
    }

    fn create_test_case_overview(custom_fields: Option<Vec<CustomFieldInfo>>) -> TestCaseOverview {
        TestCaseOverview {
            id: 1,
            project_id: 1,
            name: "Some name".to_string(),
            description: None,
            precondition: None,
            custom_fields: custom_fields,
            expected_result: None,
            tags: None
        }
    }

    fn create_tags(names: &[&str]) -> Vec<Tag> {
        names.iter()
            .map(|name| Tag {
                id: 1,
                name: name.to_string(),
            })
            .collect()
    }

    #[test]
    fn test_empty_fields() {
        let test_case_overview = create_test_case_overview(None);
        let result = test_case_overview.generate_allure_decorators_from_fields();
        assert!(result.is_empty());
    }

    #[test]
    fn test_known_meta_types() {
        let fields = vec![
            create_custom_field("Epic", "Auth"),
            create_custom_field("Feature", "Login"),
            create_custom_field("Story", "OAuth2"),
            create_custom_field("Suite", "Smoke"),
        ];
        
        let test_case_overview = create_test_case_overview(Some(fields));
        let result = test_case_overview.generate_allure_decorators_from_fields();
        
        assert_eq!(result, vec![
            "@allure.epic('Auth')",
            "@allure.feature('Login')",
            "@allure.story('OAuth2')",
            "@allure.suite('Smoke')",
        ]);
    }

    #[test]
    fn test_unknown_meta_type() {
        let fields = vec![
            create_custom_field("Severity", "High"),
            create_custom_field("Owner", "QA"),
        ];
       
        let test_case_overview = create_test_case_overview(Some(fields)); 
        let result = test_case_overview.generate_allure_decorators_from_fields();
        
        assert_eq!(result, vec![
            "@allure.label('severity', 'High')",
            "@allure.label('owner', 'QA')",
        ]);
    }

    #[test]
    fn test_mixed_types() {
        let fields = vec![
            create_custom_field("Epic", "Core"),
            create_custom_field("Priority", "P0"),
            create_custom_field("Story", "API"),
        ];
       
        let test_case_overview = create_test_case_overview(Some(fields));  
        let result = test_case_overview.generate_allure_decorators_from_fields();
        
        assert_eq!(result, vec![
            "@allure.epic('Core')",
            "@allure.label('priority', 'P0')",
            "@allure.story('API')",
        ]);
    }

    #[test]
    fn test_concat_all_description_empty() {
        let test_case_overview = TestCaseOverview {
            description: None,
            precondition: None,
            expected_result: None,
            ..create_test_case_overview(None)
        };
        assert_eq!(test_case_overview.concat_all_description(), "");
    }

    #[test]
    fn test_concat_all_description_single_field() {
        let test_case_overview = TestCaseOverview {
            description: Some("Test description".into()),
            precondition: None,
            expected_result: None,
            ..create_test_case_overview(None)
        };
        assert_eq!(test_case_overview.concat_all_description(), "Test description");
    }

    #[test]
    fn test_concat_all_description_multiple_fields() {
        let test_case_overview = TestCaseOverview {
            description: Some("First part".into()),
            precondition: Some("Second part".into()),
            expected_result: Some("Third part".into()),
            ..create_test_case_overview(None)
        };
        assert_eq!(
            test_case_overview.concat_all_description(),
            "First part\n\nSecond part\n\nThird part"
        );
    }

    #[test]
    fn test_concat_with_missing_middle_field() {
        let test_case_overview = TestCaseOverview {
            description: Some("Start".into()),
            precondition: None,
            expected_result: Some("End".into()),
            ..create_test_case_overview(None)
        };
        assert_eq!(test_case_overview.concat_all_description(), "Start\n\nEnd");
    }

    #[test]
    fn test_newline_replacement() {
        let test_case_overview = TestCaseOverview {
            description: Some("Line1\nLine2".into()),
            precondition: Some("Step1\nStep2".into()),
            expected_result: None,
            ..create_test_case_overview(None)
        };
        assert_eq!(
            test_case_overview.concat_all_description(),
            "Line1\nLine2\n\nStep1\nStep2"
        );
    }

    #[test]
    fn test_all_fields_with_special_chars() {
        let test_case_overview = TestCaseOverview {
            description: Some("Desc: \n\t@".into()),
            precondition: Some("Pre: ~!@".into()),
            expected_result: Some("Result: %^&".into()),
            ..create_test_case_overview(None)
        };
        assert_eq!(
            test_case_overview.concat_all_description(),
            "Desc: \n\t@\n\nPre: ~!@\n\nResult: %^&"
        );
    }

    #[test]
    fn test_convert_empty_metadata() {
        let test_case = create_test_case_overview(None);
        let result = test_case.convert_allure_metadata_to_python_template();
        assert_eq!(result, "");
    }

    #[test]
    fn test_convert_with_only_tags() {
        let mut test_case = create_test_case_overview(None);
        test_case.tags = Some(create_tags(&["smoke", "regression"]));
        
        let result = test_case.convert_allure_metadata_to_python_template();
        assert_eq!(result, "@allure.tag('smoke', 'regression')");
    }

    #[test]
    fn test_convert_with_fields_and_tags() {
        let fields = vec![
            create_custom_field("Epic", "Auth"),
            create_custom_field("Feature", "Login"),
        ];
        let mut test_case = create_test_case_overview(Some(fields));
        test_case.tags = Some(create_tags(&["api", "security"]));
        
        let result = test_case.convert_allure_metadata_to_python_template();
        assert_eq!(
            result,
            "@allure.epic('Auth')\n@allure.feature('Login')\n@allure.tag('api', 'security')"
        );
    }

    #[test]
    fn test_special_characters_in_tags() {
        let mut test_case = create_test_case_overview(None);
        test_case.tags = Some(create_tags(&["data's", "\"quoted\""]));
        
        let result = test_case.convert_allure_metadata_to_python_template();
        assert_eq!(
            result,
            "@allure.tag('data's', '\"quoted\"')"
        );
    }

    #[test]
    fn test_empty_tags() {
        let mut test_case = create_test_case_overview(None);
        test_case.tags = Some(vec![]);
        
        let result = test_case.convert_allure_metadata_to_python_template();
        assert_eq!(result, "");
    }
}