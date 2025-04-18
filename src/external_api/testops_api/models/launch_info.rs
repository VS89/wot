#[derive(serde::Serialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LaunchInfo {
    name: String,
    project_id: u32,
}

impl LaunchInfo {
    #[cfg(test)]
    pub fn default() -> Self {
        Self {
            name: "test_report_upload".to_string(),
            project_id: 2,
        }
    }

    pub fn new(name: &str, project_id: u32) -> Self {
        Self {
            name: name.to_string(),
            project_id,
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_create_launch_info() {
        assert_eq!(
            LaunchInfo::new("my_launch", 12345),
            LaunchInfo {
                name: "my_launch".to_string(),
                project_id: 12345
            }
        );
    }

    #[test]
    fn test_default_launch_info() {
        assert_eq!(
            LaunchInfo::default(),
            LaunchInfo {
                name: "test_report_upload".to_string(),
                project_id: 2
            }
        );
    }
}
