pub mod cli_app;
pub mod command_logic;
pub mod config;
pub mod constants;
pub mod errors;
pub mod external_api;
pub mod utils;

use constants::CONFIG_DIR;
use directories::UserDirs;

use external_api::base_api_client::ApiError;
use external_api::testops_api::testops_api::TestopsApi;
use external_api::testops_api::models::{test_case_scenario::Scenario, test_case_overview::TestCaseOverview};
use utils::{get_file_name, read_file_to_buffer};
use std::collections::HashSet;
use std::fs::{self, read_dir, File};
use std::io::Write;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use zip::{write::SimpleFileOptions, CompressionMethod, ZipWriter};

/// Get path directory with report tests
fn get_dir_archive() -> Result<PathBuf, ApiError> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| ApiError::InvalidSystemTime)?
        .as_secs();

    UserDirs::new()
        .ok_or(ApiError::NotFoundUserDir)
        .map(|user_dirs| {
            user_dirs.home_dir()
                .join(CONFIG_DIR)
                .join(format!("testops_results_report_{timestamp}.zip"))
        })
}

/// Validate project id. Check project_id is in project ids list testops
async fn validate_project_id(
    project_id: u32,
    testops_api_client: &TestopsApi,
) -> Result<(), ApiError> {
    let set_project_ids: HashSet<u32> = testops_api_client.get_all_project_ids().await?;
    if !set_project_ids.contains(&project_id){
        return Err(ApiError::ProjectIdNotFound(project_id));
    }
    Ok(())
}

/// Directory archive to *.zip
pub async fn zip_directory(path_to_report_dir: &str) -> Result<PathBuf, ApiError> {
    let dir_archive = get_dir_archive()?;
    if let Some(parent) = &dir_archive.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut zip = ZipWriter::new(File::create(&dir_archive)?);

    // Настройки для файла в архиве
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .unix_permissions(0o755);
    let entries = read_dir(path_to_report_dir)
        .map_err(|_| ApiError::NotFoundDirByPath(path_to_report_dir.to_string()))?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let buffer = read_file_to_buffer(&path).await?;
        let file_name = get_file_name(&path)?;

        zip.start_file(file_name, options)?;
        let _ = zip.write_all(&buffer);
    }
    // Завершаем запись архива
    zip.finish()?;
    Ok(dir_archive)
}

/// Create file in current directory
///
/// Return full path to created file
fn create_file_in_current_directory(file_name: &str, content: &[u8]) -> Result<String, ApiError> {
    let mut file = File::create(file_name).map_err(|_| ApiError::CouldNotCreateFile)?;
    let _ = file.write_all(content);
    let mut path = std::env::current_dir().map_err(|_| ApiError::CouldNotCreateFile)?;
    path.push(file_name);
    Ok(path.display().to_string())
}


pub fn create_template_python_ati_su(test_case_overview: TestCaseOverview,
    test_case_scenario: Scenario, file_name: &str) -> Result<String, ApiError>{
    let allure_metadata = test_case_overview.convert_allure_metadata_to_python_template();
    let all_description = test_case_overview.concat_all_description();
    let scenario = test_case_scenario.get_scenario();
    let template = format!(
        "import pytest
import allure


{}
@pytest.mark.TEMPLATE_MARK_NAME
class Test1:

    @allure.id('{}')
    @allure.title('{}')
    def test1(self):
        \"\"\"
        {}

        {}

        Шаги:
            {}
        \"\"\"
        pass
",
        allure_metadata,
        test_case_overview.id,
        test_case_overview.name,
        test_case_overview.name,
        all_description,
        scenario,
    );
    create_file_in_current_directory(file_name, template.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;
    use external_api::testops_api::testops_api::TestopsApi;
    use external_api::testops_api::models::launch_info::LaunchInfo;

    #[tokio::test]
    async fn test_validate_project_id_exist() {
        let testops_api = TestopsApi::default();
        let project_id_exist: u32 = 2;
        let res = validate_project_id(project_id_exist, &testops_api)
            .await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_validate_project_id_nonexist() {
        let testops_api = TestopsApi::default();
        let project_id_nonexist: u32 = 28888;
        let res = validate_project_id(project_id_nonexist, &testops_api)
            .await;
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            format!("Project with ID == {} not found", project_id_nonexist)
        );
    }

    #[test]
    /// Получение директории до файла с архивом результатов
    fn test_get_dir_archive_path() {
        let dir_path = get_dir_archive().unwrap_or(PathBuf::new());
        let re = Regex::new(r"testops_results_report_\d+\.zip").unwrap();
        let archive_file_name = dir_path
            .file_name()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default();
        if !re.is_match(archive_file_name) {
            assert!(
                false,
                "Получили dir_path == \"{}\". Ожидали что имя файла будет соответствовать шаблону testops_results_report_\\d+\\.zip",
                archive_file_name
            );
        }
    }

    #[tokio::test]
    /// Проверяем архивацию пустой папки
    async fn test_zip_empty_directory() {
        // Получаем полный путь до папки для архивирования и создаем ее(пустая без файлов)
        let binding = UserDirs::new().unwrap();
        let desktop_dir = binding.desktop_dir().unwrap();
        let full_path: PathBuf = desktop_dir.join("dir_for_test_zip_directory");
        let _ = fs::create_dir_all(&full_path);
        // Используем функцию zip_directory и проверяем полученный путь
        let zip_dir = zip_directory(&full_path.to_str().unwrap()).await.unwrap_or_default();
        let re = Regex::new(r"testops_results_report_\d+\.zip").unwrap();
        if !re.is_match(&zip_dir.to_str().unwrap_or_default()) {
            assert!(
                false,
                "Получили dir_path == <{:?}>. Ожидали что имя файла будет соответствовать шаблону testops_results_report_\\d+\\.zip",
                &zip_dir.to_str().unwrap_or_default()
            );
        }
    }

    #[tokio::test]
    /// Проверяем архивацию директории, в которой есть файл
    async fn test_zip_dir_with_one_file() {
        // Получаем полный путь до папки для архивирования и создаем ее(пустая без файлов)
        let binding = UserDirs::new().unwrap();
        let desktop_dir = binding.desktop_dir().unwrap();
        let full_path: PathBuf = desktop_dir.join("dir_for_test_zip_directory_with_one_file");
        let _ = fs::create_dir_all(&full_path);
        // Создаем файл в директории
        let _ = File::create(full_path.join("some_file.json")).unwrap();
        // Используем функцию zip_directory и проверяем полученный путь
        let zip_dir = zip_directory(&full_path.to_str().unwrap()).await.unwrap_or_default();
        let re = Regex::new(r"testops_results_report_\d+\.zip").unwrap();
        if !re.is_match(&zip_dir.to_str().unwrap_or_default()) {
            assert!(
                false,
                "Получили dir_path == <{:?}>. Ожидали что имя файла будет соответствовать шаблону testops_results_report_\\d+\\.zip",
                &zip_dir.to_str().unwrap_or_default()
            );
        }
    }

    #[tokio::test]
    /// Проверяем загрузку лаунча
    async fn test_upload_launch() {
        let testops_api_client = TestopsApi::default();
        let launch_info = LaunchInfo::default();
        let path_archive = PathBuf::from(format!(
            "{}/test_files/testops_results_report_1735389182.zip",
            env!("CARGO_MANIFEST_DIR")
        ));
        let _ = testops_api_client.post_upload_report(&path_archive, &launch_info)
            .await
            .unwrap();
    }

    #[test]
    fn test_scenario_parse() {
        let data = fs::read_to_string(format!("{}/test_files/scenario_with_expected_result.json", env!("CARGO_MANIFEST_DIR"))).unwrap();
        let scenario: Scenario = serde_json::from_str(&data).expect("Ошибка парсинга JSON из файла /test_files/scenario_with_expected_result.json");
        let exp_str = "Подготовка к тесту
\t\t\t\tПроверка после подготовки
\t\t\tВторой шаг, что то дергаем
\t\t\tЗавершаем тест
\t\t\tДобавили еще один шаг
\t\t\tПервый шаг, создаем юзера
\t\t\t\tПроверяем 200 и user_id не пустой
\t\t\t\tи что нибудь еще
\t\t\t\tИ тут нужна еще одна проверка";
        assert_eq!(scenario.get_scenario(), exp_str);
    }

    #[test]
    fn test_empty_scenario_parse() {
        let json_str = r#"
        {
            "root": {
            "children": []
            },
            "scenarioSteps": {},
            "attachments": {},
            "sharedSteps": {},
            "sharedStepScenarioSteps": {},
            "sharedStepAttachments": {}
        }
        "#;
        let data: Scenario = serde_json::from_str(json_str).unwrap();
        let exp_str = "";
        assert_eq!(data.get_scenario(), exp_str);
    }

    #[test]
    fn test_create_template_overview_and_scenario() {
        let data_scenario = fs::read_to_string(format!("{}/test_files/scenario_with_expected_result.json", env!("CARGO_MANIFEST_DIR"))).unwrap();
        let scenario: Scenario = serde_json::from_str(&data_scenario).expect("Ошибка парсинга JSON из файла /test_files/scenario_with_expected_result.json");
        let data_overview = fs::read_to_string(format!("{}/test_files/test_case_overview_24442.json", env!("CARGO_MANIFEST_DIR"))).unwrap();
        let overview: TestCaseOverview = serde_json::from_str(&data_overview).expect("Ошибка парсинга JSON из файла /test_files/test_case_overview_24442.json");
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let file_name = format!("test_{}_{}.py", timestamp, 24442);
        let _ = create_template_python_ati_su(overview, scenario, &file_name);
    }
}
