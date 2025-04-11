#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct ResponseLaunchUpload {
    #[serde(rename = "launchId")]
    pub launch_id: u32,
    #[serde(rename = "testSessionId")]
    test_session_id: u32,
    #[serde(rename = "filesCount")]
    files_count: u32,
}

impl ResponseLaunchUpload {

    #[cfg(test)]
    pub fn default() -> Self {
        Self { launch_id: 11111, test_session_id: 1, files_count: 10 }
    }
}