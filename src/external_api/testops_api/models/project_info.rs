#[derive(serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ProjectInfo {
    pub id: u32,
    pub name: String,
}