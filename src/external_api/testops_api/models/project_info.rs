#[derive(serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ProjectInfo {
    pub id: u32,
    pub name: String,
}

impl ProjectInfo {

    #[cfg(test)]
    pub fn new(id: u32, name: &str) -> Self {
        Self { id, name: name.to_string() }
    }
}