use super::project_info::ProjectInfo;

#[derive(serde::Deserialize,  serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ResponseGetAllProject {
    pub total_pages: u32,
    pub content: Vec<ProjectInfo>,
}
