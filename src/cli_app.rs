use clap::{Args, Parser, Subcommand};

use crate::external_api::{testops_api::TestopsApi, ApiError};
use crate::{send_report, import_testcase_by_id};

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
    #[arg(long, short, required = true, value_parser = validate_u32_more_then_zero)]
    pub import_testcase_id: u32,
}

fn validate_u32_more_then_zero(value: &str) -> Result<u32, ApiError> {
    let project_id: u32 = value.parse().map_err(|_| ApiError::Parse(value.to_string()))?;
    if project_id == 0 {
        return Err(ApiError::ProjectIdMoreThenZero)
    }
    Ok(project_id)
}


pub async fn handle_command(
    cli: Cli,
    testops_api: &TestopsApi,
    stdin: std::io::Stdin,
    stdout: std::io::Stdout,
) {
    match &cli.command {
        Commands::Report(value) => {
            match send_report(&value.directory_path, value.project_id, testops_api, stdin.lock(), stdout).await {
                Ok(value) => println!("{}", value),
                Err(e) => eprintln!("Failed to send report: {}", e),
            };
        }
        Commands::Testcase(value) => {
            match import_testcase_by_id(value.import_testcase_id, testops_api).await {
                Ok(value) => println!("{}", value),
                Err(e) => eprintln!("Failed to import testcase by id: {}", e),
            };
        }
    }
}


#[cfg(test)]
mod tests {

    use super::*;
    use clap::Parser;
    use rstest::rstest;

    #[rstest]
    #[case("some_dir", "-d")]
    #[case("./some_dir", "--directory-path")]
    #[case("./one_dir/second_dir", "-d")]
    fn test_report_command_positive(#[case] dir_path: String, #[case] flag: String) {
        let args = Cli::parse_from(["wot", "report", &flag, &dir_path, "-p", "777"]);
        match args.command {
            Commands::Report(value) => {
                assert_eq!(value.directory_path, dir_path);
                assert_eq!(value.project_id, 777);
            },
            _ => {}
        }
    }

    #[rstest]
    #[case("-i")]
    #[case("--import-testcase-id")]
    fn test_import_testcase_command_positive(#[case] flag: String) {
        let args = Cli::parse_from(["wot", "testcase", &flag, "1111"]);
        match args.command {
            Commands::Testcase(value) => {
                assert_eq!(value.import_testcase_id, 1111);
            },
            _ => {}
        }
    }

    #[test]
    fn test_validate_value_equal_zero() {
        let result = validate_u32_more_then_zero("0");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ApiError::ProjectIdMoreThenZero));
    }

    #[rstest]
    #[case("")]
    #[case("@")]
    #[case("-1")]
    #[case("4294967296")]
    fn test_validate_value_parse_error(#[case] value: String) {
        let result = validate_u32_more_then_zero(&value);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ApiError::Parse(_)));
    }

    #[rstest]
    #[case("-h")]
    #[case("--help")]
    fn test_help_output(#[case] flag: String) {
        let exp_help_text = r#"CLI application for Allure TestOps <https://qameta.io/>. wot - WrapperOverTestops

Usage: wot <COMMAND>

Commands:
  report    Uploading a report to TestOps
  testcase  Action with testcase
  help      Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
"#;
        let mut cmd = assert_cmd::Command::cargo_bin("wot").unwrap();
        cmd.arg(flag)
            .assert()
            .success()
            .stdout(predicates::str::contains(exp_help_text));
    }

    #[rstest]
    #[case("-h")]
    #[case("--help")]
    fn test_report_help_output(#[case] flag: String) {
        let exp_help_text = r#"Uploading a report to TestOps

Usage: wot report --directory-path <DIRECTORY_PATH> --project-id <PROJECT_ID>

Options:
  -d, --directory-path <DIRECTORY_PATH>  Path to directory
  -p, --project-id <PROJECT_ID>          Allure project id
  -h, --help                             Print help
  -V, --version                          Print version
"#;
        let mut cmd = assert_cmd::Command::cargo_bin("wot").unwrap();
        cmd.args(["report", &flag])
            .assert()
            .success()
            .stdout(predicates::str::contains(exp_help_text));
    }

    #[rstest]
    #[case("-h")]
    #[case("--help")]
    fn test_testcase_help_output(#[case] flag: String) {
        let exp_help_text = r#"Action with testcase

Usage: wot testcase --import-testcase-id <IMPORT_TESTCASE_ID>

Options:
  -i, --import-testcase-id <IMPORT_TESTCASE_ID>  Import testcase
  -h, --help                                     Print help
  -V, --version                                  Print version
"#;
        let mut cmd = assert_cmd::Command::cargo_bin("wot").unwrap();
        cmd.args(["testcase", &flag])
            .assert()
            .success()
            .stdout(predicates::str::contains(exp_help_text));
    }

    #[rstest]
    #[case("report")]
    #[case("testcase")]
    fn test_missing_required_args(#[case] flag: String) {
        let mut cmd = assert_cmd::Command::cargo_bin("wot").unwrap();
        cmd.arg(flag)
            .assert()
            .failure()
            .stderr(predicates::str::contains("required"));
    }
}