pub const CONFIG_DIR: &str = ".config/wot";
pub const ENTER_INSTANCE_URL_TESTOPS: &str = "Enter the url of the testops instance: ";
pub const ENTER_TESTOPS_API_KEY: &str = "Enter the TestOps API key: ";
pub const COMPLETE_SETUP: &str = "To view the available commands, type: wot --help";

/// Standard message
#[derive(Debug)]
pub enum Message {
    LaunchRunFrom(String),
    LaunchLinkDownloaded(String, String),
    ApproveUploadReport(String),
}

impl Message {
    pub fn to_formatted_string(&self) -> String {
        match self {
            Message::LaunchRunFrom(name) => format!("Run from {}", name),
            Message::LaunchLinkDownloaded(testops_instance, launch_id) => format!(
                "Link to downloaded lunch: {}launch/{}",
                testops_instance, launch_id
            ),
            Message::ApproveUploadReport(value) => format!(
                "You want to load a report into a project: '{}' [y/n]? ",
                value
            ),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::TestopsApi;
    use rstest::rstest;

    fn get_testops_instance_url() -> String {
        TestopsApi::default().client.base_url.to_string()
    }

    #[rstest]
    #[case("SomeName", "Run from SomeName")]
    #[case("", "Run from ")]
    #[case("Some&Name", "Run from Some&Name")]
    #[case("a".repeat(1000), format!("Run from {}", "a".repeat(1000)))]
    fn test_launch_run_from(#[case] input: String, #[case] expected: String) {
        assert_eq!(Message::LaunchRunFrom(input).to_formatted_string(), expected);
    }

    #[rstest]
    #[case(get_testops_instance_url(), "12345", format!("Link to downloaded lunch: {}launch/12345", get_testops_instance_url()))]
    #[case("", "", "Link to downloaded lunch: launch/")]
    #[case("a".repeat(1000), "a".repeat(1000), format!("Link to downloaded lunch: {}launch/{}", "a".repeat(1000), "a".repeat(1000)))]
    fn test_launch_link_downloaded(#[case] instance: String, #[case] launch_id: String, #[case] expected: String) {
        assert_eq!(
            Message::LaunchLinkDownloaded(instance, launch_id).to_formatted_string(),
            expected
        );
    }

    #[rstest]
    #[case("TestProject", "You want to load a report into a project: 'TestProject' [y/n]? ")]
    #[case("", "You want to load a report into a project: '' [y/n]? ")]
    #[case("Test&Project", "You want to load a report into a project: 'Test&Project' [y/n]? ")]
    #[case("a".repeat(1000), format!("You want to load a report into a project: '{}' [y/n]? ", "a".repeat(1000)))]
    fn test_approve_upload_report(#[case] input: String, #[case] expected: String) {
        assert_eq!(
            Message::ApproveUploadReport(input).to_formatted_string(),
            expected
        );
    }
}
