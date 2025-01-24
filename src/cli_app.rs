use clap::{Args, Parser, Subcommand};

use crate::errors::WotError;

#[derive(Parser)]
#[command(
    name = "wot",
    version = "0.1.0",
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
}

#[derive(Args)]
pub struct ReportArgs {
    /// Path to directory
    #[arg(long, short, required = true)]
    pub directory_path: String,
    /// Allure project id
    #[arg(long, short, required = true, value_parser = validate_project_id)]
    pub project_id: u32,
}

// Валидация параметра project_id. В целом простую валидацю параметра описывать не обязательно,
// ее отлично выполняет clap
// Надо бы еще прикрутить, чтобы тут брались id проектов из тестопса и показывало какие данные можно вводить
fn validate_project_id(value: &str) -> Result<u32, WotError> {
    let project_id: u32 = value.parse().map_err(|_| WotError::ProjectIdMoreThenZero)?;
    Ok(project_id)
}
