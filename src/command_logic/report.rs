use crate::constants::Message;

use crate::external_api::testops_api::models::launch_info::LaunchInfo;
use crate::external_api::testops_api::models::response_launch_upload::ResponseLaunchUpload;
use crate::external_api::testops_api::TestopsApi;
use crate::external_api::ApiError;
use crate::utils::{validate_project_id, zip_directory};
use std::fs;
use std::io::{BufRead, Write};

/// Sending report to TestOps
pub async fn send_report<R, W>(
    path_to_report_directory: &str,
    project_id: u32,
    testops_api_client: &TestopsApi,
    input: R,
    output: W,
) -> Result<String, ApiError>
where
    R: BufRead,
    W: Write,
{
    validate_project_id(project_id, testops_api_client).await?;
    confirm_upload_to_project(project_id, testops_api_client, input, output).await?;
    let result = zip_directory(path_to_report_directory).await?;
    let generate_launch_name = chrono::Local::now().format("%d/%m/%Y %H:%M").to_string();
    let launch_info = LaunchInfo::new(
        &Message::LaunchRunFrom(generate_launch_name).to_formatted_string(),
        project_id,
    );
    let response: ResponseLaunchUpload = testops_api_client
        .post_upload_report(&result, &launch_info)
        .await?;
    let _ = fs::remove_file(&result);
    Ok(Message::LaunchLinkDownloaded(
        testops_api_client.client.base_url.to_string(),
        response.launch_id.to_string(),
    )
    .to_formatted_string())
}

/// Confirm upload to project
async fn confirm_upload_to_project<R, W>(
    project_id: u32,
    testops_api_client: &TestopsApi,
    mut input: R,
    mut output: W,
) -> Result<(), ApiError>
where
    R: BufRead,
    W: Write,
{
    let project_info = testops_api_client
        .get_project_info_by_id(&project_id)
        .await?;

    output.write_all(
        Message::ApproveUploadReport(project_info.name)
            .to_formatted_string()
            .as_bytes(),
    )?;
    output.flush()?;

    let mut confirmation = String::new();
    input.read_line(&mut confirmation)?;

    let trimmed = confirmation.trim().to_lowercase();
    if !matches!(trimmed.as_str(), "y" | "yes" | "") {
        return Err(ApiError::UploadCancelledByUser);
    }
    Ok(())
}

#[cfg(test)]
mod tests {

    use crate::external_api::testops_api::models::project_info::ProjectInfo;

    use super::*;
    use crate::constants::CARGO_MANIFEST_DIR;
    use mockito::ServerGuard;
    use rstest::rstest;
    use std::path::Path;
    use std::{
        io::{BufReader, Cursor},
        path::PathBuf,
    };

    async fn precondition_send_report(
        path_to_report: PathBuf,
        mock_response_launch_upload: &ResponseLaunchUpload,
        mut server: &mut ServerGuard,
    ) -> Result<String, ApiError> {
        let mock_response = ProjectInfo {
            id: 2,
            name: "Test Project".to_string(),
        };

        let testops_api = TestopsApi::mock(&server.url());
        TestopsApi::mock_get_all_projects(&mut server).await;
        TestopsApi::mock_post_upload_report(&mut server, &mock_response_launch_upload).await;
        TestopsApi::mock_get_project_by_id(
            &mut server,
            &ProjectInfo {
                id: 2,
                name: "Test Project".to_string(),
            },
        )
        .await;

        let input = BufReader::new(Cursor::new(b"y".to_vec()));
        let mut output = Cursor::new(Vec::<u8>::new());

        send_report(
            path_to_report.to_str().unwrap(),
            mock_response.id,
            &testops_api,
            input,
            &mut output,
        )
        .await
    }

    #[tokio::test]
    #[rstest]
    #[case(b"\n")]
    #[case(b"y\n")]
    #[case(b"yes\n")]
    #[case(b"")]
    #[case(b"y")]
    #[case(b"yes")]
    async fn test_confirm_project(#[case] input_data: &[u8]) {
        let mock_response = ProjectInfo {
            id: 2,
            name: "Test Project".to_string(),
        };
        let mut server = mockito::Server::new_async().await;

        let testops_api = TestopsApi::mock(&server.url());
        TestopsApi::mock_get_project_by_id(&mut server, &mock_response).await;

        let input = BufReader::new(Cursor::new(input_data.to_vec()));

        let mut output = Cursor::new(Vec::<u8>::new());
        let result =
            confirm_upload_to_project(mock_response.id, &testops_api, input, &mut output).await;
        assert!(result.is_ok());

        let output_str = String::from_utf8(output.into_inner()).unwrap();
        assert_eq!(
            output_str,
            Message::ApproveUploadReport("Test Project".to_string()).to_formatted_string()
        );
    }

    #[tokio::test]
    #[rstest]
    #[case(b"n\n")]
    #[case(b"yse\n")]
    async fn test_not_confirm_project(#[case] input_data: &[u8]) {
        let mock_response = ProjectInfo {
            id: 2,
            name: "Test Project".to_string(),
        };
        let mut server = mockito::Server::new_async().await;

        let testops_api = TestopsApi::mock(&server.url());
        TestopsApi::mock_get_project_by_id(
            &mut server,
            &ProjectInfo {
                id: 2,
                name: "Test Project".to_string(),
            },
        )
        .await;

        let input = BufReader::new(Cursor::new(input_data.to_vec()));

        let mut output = Cursor::new(Vec::<u8>::new());
        let result =
            confirm_upload_to_project(mock_response.id, &testops_api, input, &mut output).await;
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            ApiError::UploadCancelledByUser.to_string()
        );
    }

    #[tokio::test]
    async fn test_send_report_error() {
        let mock_response_launch_upload = ResponseLaunchUpload::default();
        let mut server = mockito::Server::new_async().await;
        let path_to_report =
            Path::new(CARGO_MANIFEST_DIR).join("test_files/test_upload_launch_report.zip");
        let send_report =
            precondition_send_report(path_to_report, &mock_response_launch_upload, &mut server)
                .await;

        assert!(send_report.is_err());
        assert!(matches!(
            send_report.unwrap_err(),
            ApiError::NotFoundDirByPath(_)
        ));
    }

    #[tokio::test]
    async fn test_send_report_success() {
        let mock_response_launch_upload = ResponseLaunchUpload::default();
        let mut server = mockito::Server::new_async().await;
        let path_to_report = Path::new(CARGO_MANIFEST_DIR).join("test_files/report_test_project");
        let result =
            precondition_send_report(path_to_report, &mock_response_launch_upload, &mut server)
                .await;

        assert!(result.is_ok());
        let exp_result = format!(
            "Link to downloaded launch: {}/launch/{}",
            server.url(),
            mock_response_launch_upload.launch_id
        );
        assert_eq!(result.unwrap(), exp_result);
    }
}
