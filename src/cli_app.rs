use clap::{Args, Parser, Subcommand};

use crate::errors::WotError;

#[derive(Parser)]
#[command(
    name = "wot",
    version = "0.2.3",
    author = "Valentin Semenov <valentin@semenov-aqa.ru>",
    about = "CLI application for Allure TestOps <https://qameta.io/>. wot - WrapperOverTestops",
    long_about = None,
    propagate_version = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Uploading a report to TestOps
    Report(ReportArgs),
    /// Action with testcase
    Testcase(TestcaseArgs),
}

#[derive(Args)]
pub struct ReportArgs {
    /// Path to directory
    #[arg(long, short, required = true)]
    pub directory_path: String,
    /// Allure project id
    #[arg(long, short, required = true, value_parser = validate_u32_more_then_zero)]
    pub project_id: u32,
}

#[derive(Args)]
pub struct TestcaseArgs {
    /// Import testcase
    #[arg(long, short, value_parser = validate_u32_more_then_zero)]
    pub import_testcase_id: u32,
}

fn validate_u32_more_then_zero(value: &str) -> Result<u32, WotError> {
    let project_id: u32 = value.parse().map_err(|_| WotError::ProjectIdMoreThenZero)?;
    Ok(project_id)
}
