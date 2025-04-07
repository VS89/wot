use super::custom_field::CustomField;

#[derive(serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CustomFieldInfo {
    pub id: u32,
    pub name: String,
    pub custom_field: CustomField,
}