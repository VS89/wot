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
                "Link to downloaded lunch: {}/launch/{}",
                testops_instance, launch_id
            ),
            Message::ApproveUploadReport(value) => format!(
                "You want to load a report into a project: '{}' [y/n]? ",
                value
            ),
        }
    }
}
