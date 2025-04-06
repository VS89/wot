#[derive(Default, Debug, PartialEq)]
pub enum AllureMetaData {
    #[default]
    Unknown,
    Epic(String),
    Feature(String),
    Story(String),
    Suite(String),
    Label(String),
}

impl AllureMetaData {

    fn create_decorator(tag: &str, value: &str) -> String {
        format!("@allure.{tag}('{value}')")
    }

    pub fn epic(value: &str) -> String {
        Self::create_decorator("epic", value)
    }

    pub fn feature(value: &str) -> String {
        Self::create_decorator("feature", value)
    }

    pub fn story(value: &str) -> String {
        Self::create_decorator("story", value)
    }

    pub fn suite(value: &str) -> String {
        Self::create_decorator("suite", value)
    }

    pub fn label(field_name: &str, value: &str) -> String {
        format!("@allure.label('{}', '{value}')", field_name.to_ascii_lowercase())
    }
}


#[cfg(test)]
mod tests {

    use super::*;
    
    #[test]
    fn test_epic_creation() {
        let decorator = AllureMetaData::epic("Login");
        assert_eq!(decorator, "@allure.epic('Login')");
    }

    #[test]
    fn test_feature_creation() {
        let decorator = AllureMetaData::feature("Auth");
        assert_eq!(decorator, "@allure.feature('Auth')");
    }

    #[test]
    fn test_story_creation() {
        let decorator = AllureMetaData::story("OAuth2");
        assert_eq!(decorator, "@allure.story('OAuth2')");
    }

    #[test]
    fn test_suite_creation() {
        let decorator = AllureMetaData::suite("Smoke");
        assert_eq!(decorator, "@allure.suite('Smoke')");
    }

    #[test]
    fn test_label_creation() {
        let decorator = AllureMetaData::label("Smoke", "ValueSmoke");
        assert_eq!(decorator, "@allure.label('smoke', 'ValueSmoke')");
    }

    #[test]
    fn test_empty_value() {
        let decorator = AllureMetaData::epic("");
        assert_eq!(decorator, "@allure.epic('')");
    }

    #[test]
    fn test_special_characters() {
        let decorator = AllureMetaData::feature("User's Dashboard");
        assert_eq!(decorator, "@allure.feature('User's Dashboard')");
    }
}