use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetLaunchByIdResponse {
    id: u32,
    name: String,
    project_id: u8
}