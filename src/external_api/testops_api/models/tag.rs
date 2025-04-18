#[derive(serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Tag {
    pub id: u32,
    pub name: String,
}
