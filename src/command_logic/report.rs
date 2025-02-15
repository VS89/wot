use crate::config::Config;
use crate::constants::Message;
use crate::errors::{FAILED_FLUSH_STDOUT, FAILED_WRITE_STDOUT};

use crate::external_api::testops::{LaunchInfo, ResponseLaunchUpload, TestopsApiClient};
use crate::{validate_project_id, zip_directory};
use std::error::Error;
use std::fs;
use std::io::Write;

/// Sending report to TestOps
pub async fn send_report(
    path_to_report_directory: &str,
    project_id: u32,
    config: &Config,
) -> Result<(), Box<dyn std::error::Error>> {
    validate_project_id(&project_id, config).await?;
    let confirm_flag = confirm_upload_to_project(&project_id, config).await?;
    if !confirm_flag {
        return Ok(());
    }
    let result = zip_directory(path_to_report_directory)?;
    let testops = TestopsApiClient::new(config);
    let generate_launch_name = chrono::Local::now().format("%d/%m/%Y %H:%M").to_string();
    let launch_info = LaunchInfo::new(
        &Message::LaunchRunFrom(generate_launch_name).to_formatted_string(),
        project_id,
    );
    let response: ResponseLaunchUpload = match testops
        .post_archive_report_launch_upload(&result, launch_info)
        .await
    {
        Ok(value) => value,
        Err(e) => {
            let _ = fs::remove_file(&result);
            return Err(e);
        }
    };
    println!(
        "{}",
        Message::LaunchLinkDownloaded(
            config.testops_base_url.clone(),
            response.launch_id.to_string()
        )
        .to_formatted_string()
    );
    let _ = fs::remove_file(&result);
    Ok(())
}

/// Confirm upload to project
async fn confirm_upload_to_project(
    project_id: &u32,
    config: &Config,
) -> Result<bool, Box<dyn Error>> {
    let testops = TestopsApiClient::new(config);
    let project_info = testops.get_project_info_by_id(project_id).await?;

    let mut stdout = std::io::stdout();
    stdout
        .write_all(
            Message::ApproveUploadReport(project_info.name)
                .to_formatted_string()
                .as_bytes(),
        )
        .expect(FAILED_WRITE_STDOUT);
    stdout.flush().expect(FAILED_FLUSH_STDOUT);

    let mut confirmantion = String::new();
    std::io::stdin().read_line(&mut confirmantion)?;

    let trim_lowercase_confirmation = confirmantion.trim().to_lowercase();
    if trim_lowercase_confirmation == "y" || trim_lowercase_confirmation == "yes" {
        Ok(true)
    } else {
        Ok(false)
    }
}
