use crate::constants::Message;


use crate::external_api::ApiError;
use crate::utils::{validate_project_id, zip_directory};
use crate::external_api::testops_api::TestopsApi;
use crate::external_api::testops_api::models::launch_info::LaunchInfo;
use crate::external_api::testops_api::models::response_launch_upload::ResponseLaunchUpload;
use std::fs;
use std::io::Write;

/// Sending report to TestOps
pub async fn send_report(
    path_to_report_directory: &str,
    project_id: u32,
    testops_api_client: &TestopsApi,
) -> Result<(), ApiError> {
    validate_project_id(project_id, testops_api_client).await?;
    let confirm_flag = confirm_upload_to_project(project_id, testops_api_client).await?;
    if !confirm_flag {
        return Ok(());
    }
    let result = zip_directory(path_to_report_directory).await?;
    let generate_launch_name = chrono::Local::now().format("%d/%m/%Y %H:%M").to_string();
    let launch_info = LaunchInfo::new(
        &Message::LaunchRunFrom(generate_launch_name).to_formatted_string(),
        project_id,
    );
    let response: ResponseLaunchUpload = testops_api_client.post_upload_report(&result, &launch_info).await?;
    println!(
        "{}",
        Message::LaunchLinkDownloaded(
            testops_api_client.client.base_url.to_string(),
            response.launch_id.to_string()
        )
        .to_formatted_string()
    );
    let _ = fs::remove_file(&result);
    Ok(())
}

/// Confirm upload to project
async fn confirm_upload_to_project(
    project_id: u32,
    testops_api_client: &TestopsApi,
) -> Result<bool, ApiError> {
    let project_info = testops_api_client.get_project_info_by_id(&project_id).await?;

    let mut stdout = std::io::stdout();
    stdout
        .write_all(
            Message::ApproveUploadReport(project_info.name)
                .to_formatted_string()
                .as_bytes(),
        )?;
    stdout.flush()?;

    let mut confirmation = String::new();
    std::io::stdin().read_line(&mut confirmation)?;

    let trimmed = confirmation.trim().to_lowercase();
    Ok(matches!(trimmed.as_str(), "y" | "yes" | ""))
}
