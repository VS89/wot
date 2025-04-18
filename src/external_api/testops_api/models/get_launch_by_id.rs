use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetLaunchByIdResponse {
    pub id: u32,
    name: String,
    project_id: u8,
}

impl GetLaunchByIdResponse {
    #[cfg(test)]
    pub fn new(id: u32, name: &str, project_id: u8) -> Self {
        Self {
            id,
            name: name.to_string(),
            project_id,
        }
    }
}
