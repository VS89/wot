#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LaunchInfo {
    name: String,
    project_id: u32,
}

impl LaunchInfo {

    pub fn default() -> Self {
        Self { name: "test_report_upload".to_string(), project_id: 2 }
    }
}