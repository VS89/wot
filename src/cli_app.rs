use clap::{Args, Parser, Subcommand};

use crate::external_api::{testops_api::TestopsApi, ApiError};
use crate::{import_testcase_by_id, send_report};

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
    #[arg(long, short, requires = "directory_path", value_parser = validate_u32_more_then_zero)]
    pub project_id: u32,
}

#[derive(Args)]
pub struct TestcaseArgs {
    /// Import testcase
    #[arg(long, short, required = true, value_parser = validate_u32_more_then_zero)]
    pub import_testcase_id: u32,
    /// Use the file name entered by the user
    #[arg(long, short, requires = "import_testcase_id", value_parser = validate_test_file_name)]
    pub filename: Option<String>,
}

impl TestcaseArgs {
    pub fn get_filename_for_test(&self) -> String {
        format!("test_case_{}.py", self.import_testcase_id)
    }
}

fn validate_u32_more_then_zero(value: &str) -> Result<u32, ApiError> {
    let project_id: u32 = value
        .parse()
        .map_err(|_| ApiError::Parse(value.to_string()))?;
    if project_id == 0 {
        return Err(ApiError::ProjectIdMoreThenZero);
    }
    Ok(project_id)
}

fn validate_test_file_name(value: &str) -> Result<String, ApiError> {
    if !(6..=120).contains(&value.len()) || !value.starts_with("test_") {
        return Err(ApiError::InvalidTestFileName(
            "len must be between 6 and 120 and start with \"test_\"".to_string(),
        ));
    }
    let is_valid_name = value
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_');
    if !is_valid_name {
        return Err(ApiError::InvalidTestFileName(
            "only lowercase Latin letters and digit are allowed for the file name".to_string(),
        ));
    }
    Ok(value.to_string())
}

pub async fn handle_command(
    cli: Cli,
    testops_api: &TestopsApi,
    stdin: std::io::Stdin,
    stdout: std::io::Stdout,
) {
    match &cli.command {
        Commands::Report(value) => {
            match send_report(
                &value.directory_path,
                value.project_id,
                testops_api,
                stdin.lock(),
                stdout,
            )
            .await
            {
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
    use rstest::{fixture, rstest};

    const INVALID_NAME_MSG_ERROR: &str =
        "only lowercase Latin letters and digit are allowed for the file name";
    const LEN_AND_PREFIX_ERROR: &str = "len must be between 6 and 120 and start with \"test_\"";
    const MAIN_HELP: &str = r#"CLI application for Allure TestOps <https://qameta.io/>. wot - WrapperOverTestops

Usage: wot <COMMAND>

Commands:
  report    Uploading a report to TestOps
  testcase  Action with testcase
  help      Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
"#;

    const REPORT_HELP: &str = r#"Uploading a report to TestOps

Usage: wot report --directory-path <DIRECTORY_PATH> --project-id <PROJECT_ID>

Options:
  -d, --directory-path <DIRECTORY_PATH>  Path to directory
  -p, --project-id <PROJECT_ID>          Allure project id
  -h, --help                             Print help
  -V, --version                          Print version
"#;
    const TESTCASE_HELP: &str = r#"Action with testcase

Usage: wot testcase [OPTIONS] --import-testcase-id <IMPORT_TESTCASE_ID>

Options:
  -i, --import-testcase-id <IMPORT_TESTCASE_ID>  Import testcase
  -f, --filename <FILENAME>                      Use the file name entered by the user
  -h, --help                                     Print help
  -V, --version                                  Print version
"#;

    #[fixture]
    fn cli_command() -> assert_cmd::Command {
        assert_cmd::Command::cargo_bin("wot").expect("Failed to find wot binary")
    }

    #[rstest]
    #[case("")]
    #[case("test_")]
    #[case("a".repeat(5))]
    #[case("a".repeat(6))]
    #[case(format!("test_{}", "a".repeat(116)))]
    fn test_invalid_file_name_test_case(#[case] file_name: String) {
        let result = validate_test_file_name(&file_name);
        assert!(result.is_err());
        assert_eq!(
            ApiError::InvalidTestFileName(LEN_AND_PREFIX_ERROR.to_string()).to_string(),
            result.unwrap_err().to_string()
        );
    }

    #[rstest]
    #[case("test_A")]
    #[case("test_some_name_@")]
    #[case("test_###")]
    #[case("test_U+30FC")]
    fn test_invalid_file_name_digit_and_letters(#[case] file_name: String) {
        let result = validate_test_file_name(&file_name);
        assert!(result.is_err());
        assert_eq!(
            ApiError::InvalidTestFileName(INVALID_NAME_MSG_ERROR.to_string()).to_string(),
            result.unwrap_err().to_string()
        );
    }

    #[rstest]
    fn test_invalid_file_name_cmd(mut cli_command: assert_cmd::Command) {
        let result = cli_command
            .arg("testcase")
            .arg("-i")
            .arg("11111")
            .arg("-f")
            .arg("test_A")
            .assert()
            .failure();
        result.stderr(predicates::str::contains(
            ApiError::InvalidTestFileName(INVALID_NAME_MSG_ERROR.to_string()).to_string(),
        ));
    }

    #[rstest]
    #[case("-f")]
    #[case("--filename")]
    fn test_import_testcase_with_filename(#[case] flag: String) {
        let filename = "test_file";
        let args = Cli::try_parse_from(["wot", "testcase", "-i", "1111", &flag, filename])
            .expect("Failed to parse arguments");
        match args.command {
            Commands::Testcase(value) => {
                assert_eq!(value.filename.unwrap(), filename);
            }
            _ => {}
        }
    }

    #[rstest]
    #[case("-f")]
    #[case("--filename")]
    fn test_filename_requires_import_testcase(
        #[case] flag: String,
        mut cli_command: assert_cmd::Command,
    ) {
        cli_command
            .arg("testcase")
            .arg(&flag)
            .arg("file_name")
            .assert()
            .failure()
            .stderr(predicates::str::contains(
                "Invalid test file name: len must be between 6 and 120 and start with \"test_\"",
            ));
    }

    #[rstest]
    #[case("some_dir", "-d")]
    #[case("./some_dir", "--directory-path")]
    #[case("./one_dir/second_dir", "-d")]
    fn test_report_command_positive(#[case] dir_path: String, #[case] flag: String) {
        let args = Cli::try_parse_from(["wot", "report", &flag, &dir_path, "-p", "777"])
            .expect("Failed to parse arguments");
        match args.command {
            Commands::Report(value) => {
                assert_eq!(value.directory_path, dir_path);
                assert_eq!(value.project_id, 777);
            }
            _ => {}
        }
    }

    #[rstest]
    #[case("-i")]
    #[case("--import-testcase-id")]
    fn test_import_testcase_command_positive(#[case] flag: String) {
        let args = Cli::try_parse_from(["wot", "testcase", &flag, "1111"])
            .expect("Failed to parse arguments");
        match args.command {
            Commands::Testcase(value) => {
                assert_eq!(value.import_testcase_id, 1111);
            }
            _ => {}
        }
    }

    #[test]
    fn test_validate_value_equal_zero() {
        let result = validate_u32_more_then_zero("0");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ApiError::ProjectIdMoreThenZero
        ));
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
    fn test_help_output(#[case] flag: String, mut cli_command: assert_cmd::Command) {
        cli_command
            .arg(flag)
            .assert()
            .success()
            .stdout(predicates::str::contains(MAIN_HELP));
    }

    #[rstest]
    #[case("-h")]
    #[case("--help")]
    fn test_report_help_output(#[case] flag: String, mut cli_command: assert_cmd::Command) {
        cli_command
            .args(["report", &flag])
            .assert()
            .success()
            .stdout(predicates::str::contains(REPORT_HELP));
    }

    #[rstest]
    #[case("-h")]
    #[case("--help")]
    fn test_testcase_help_output(#[case] flag: String, mut cli_command: assert_cmd::Command) {
        cli_command
            .args(["testcase", &flag])
            .assert()
            .success()
            .stdout(predicates::str::contains(TESTCASE_HELP));
    }

    #[rstest]
    #[case("report")]
    #[case("testcase")]
    fn test_missing_required_args(#[case] flag: String, mut cli_command: assert_cmd::Command) {
        cli_command
            .arg(flag)
            .assert()
            .failure()
            .stderr(predicates::str::contains("required"));
    }
}
