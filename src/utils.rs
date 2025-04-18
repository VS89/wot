use std::io::Cursor;
use std::path::Path;
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt},
};
use zip::ZipArchive;

use super::external_api::ApiError;

use super::constants::CONFIG_DIR;
use directories::UserDirs;

use super::external_api::testops_api::TestopsApi;
use std::collections::HashSet;
use std::fs::{self, read_dir};
use std::io::Write;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use zip::{write::SimpleFileOptions, CompressionMethod, ZipWriter};

pub async fn read_file_to_buffer(path: &Path) -> Result<Vec<u8>, ApiError> {
    let mut file = File::open(path).await?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).await?;
    Ok(buffer)
}

/// Get file name with extension
pub fn get_file_name(path: &Path) -> Result<String, ApiError> {
    if path.is_dir() {
        return Err(ApiError::InvalidFileName);
    }
    Ok(path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or(ApiError::InvalidFileName)?
        .to_string())
}

pub fn validate_zip_archive(buffer: &Vec<u8>) -> Result<(), ApiError> {
    let cursor = Cursor::new(buffer);
    if ZipArchive::new(cursor).is_err() {
        return Err(ApiError::InvalidFileFormat);
    }
    Ok(())
}

/// Create file in current directory
///
/// Return full path to created file
pub async fn save_file_in_current_directory(
    file_name: &str,
    content: &[u8],
) -> Result<String, ApiError> {
    let mut file = File::create(file_name)
        .await
        .map_err(|_| ApiError::CouldNotCreateFile)?;
    file.write_all(content)
        .await
        .map_err(|_| ApiError::CouldNotCreateFile)?;
    let mut path = std::env::current_dir().map_err(|_| ApiError::CouldNotCreateFile)?;
    path.push(file_name);
    Ok(path.display().to_string())
}

/// Get path directory with report tests
fn get_dir_archive() -> Result<PathBuf, ApiError> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| ApiError::InvalidSystemTime)?
        .as_secs();

    UserDirs::new()
        .ok_or(ApiError::NotFoundUserDir)
        .map(|user_dirs| {
            user_dirs
                .home_dir()
                .join(CONFIG_DIR)
                .join(format!("testops_results_report_{timestamp}.zip"))
        })
}

/// Validate project id. Check project_id is in project ids list testops
pub async fn validate_project_id(
    project_id: u32,
    testops_api_client: &TestopsApi,
) -> Result<(), ApiError> {
    let set_project_ids: HashSet<u32> = testops_api_client.get_all_project_ids().await?;
    if !set_project_ids.contains(&project_id) {
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
    let mut zip = ZipWriter::new(std::fs::File::create(&dir_archive)?);

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

/// Convert to PascalCase
///
/// input - some_name
/// return - SomeName
pub fn convert_to_pascal_case(input: &str) -> String {
    input
        .split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::external_api::testops_api::models::launch_info::LaunchInfo;
    use crate::external_api::testops_api::models::test_case_scenario::Scenario;
    use crate::external_api::testops_api::TestopsApi;
    use regex::Regex;
    use rstest::rstest;
    use std::{env, fs::Permissions, os::unix::fs::PermissionsExt};
    use tokio::fs;

    const CARGO_MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR");

    fn assert_error_get_file_name(path: &Path) {
        let result = get_file_name(path);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(&error, ApiError::InvalidFileName));
        assert_eq!(error.to_string(), "Invalid file name");
    }

    fn assert_file_name(path: &Path, exp_file_name: &str) {
        let result = get_file_name(path);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), exp_file_name);
    }

    fn assert_io_error(result: Result<Vec<u8>, ApiError>) {
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(&error, ApiError::Io(_)));
        assert!(error.to_string().starts_with("IO error: "));
    }

    #[tokio::test]
    async fn test_valid_zip_archive() {
        let path = &Path::new(CARGO_MANIFEST_DIR).join("test_files/test_upload_launch_report.zip");
        let buffer = read_file_to_buffer(path).await.unwrap();
        let is_zip_file = validate_zip_archive(&buffer);
        assert!(is_zip_file.is_ok());
    }
    // Invalid file format: File is not a valid ZIP archive

    #[tokio::test]
    async fn test_invalid_zip_archive() {
        let path = &Path::new(CARGO_MANIFEST_DIR).join("test_files/file.json");
        let buffer = read_file_to_buffer(path).await.unwrap();
        let is_zip_file = validate_zip_archive(&buffer);
        assert!(is_zip_file.is_err());
        assert_eq!(
            is_zip_file.unwrap_err().to_string(),
            "Invalid file format: File is not a valid ZIP archive".to_string()
        );
    }

    #[rstest]
    #[case("test_some_one", "TestSomeOne")]
    #[case("", "")]
    #[case("test_one.py", "TestOne.py")]
    fn test_convert_to_pascal_case(#[case] filename: String, #[case] exp_pascal_case: String) {
        assert_eq!(exp_pascal_case, convert_to_pascal_case(&filename))
    }

    #[test]
    fn test_get_file_name() {
        let path = &Path::new(CARGO_MANIFEST_DIR).join("test_files/file.json");
        assert_file_name(path, "file.json");
    }

    #[test]
    fn test_get_file_name_directory_path() {
        let path = &Path::new(CARGO_MANIFEST_DIR).join("test_files/");
        assert_error_get_file_name(path);
    }

    #[test]
    fn test_get_file_name_file_without_extension() {
        let path = &Path::new(CARGO_MANIFEST_DIR).join("test_files/file_without_extension");
        assert_file_name(path, "file_without_extension");
    }

    #[test]
    fn test_get_file_name_epmty_path() {
        let path = Path::new("");
        assert_error_get_file_name(path);
    }

    #[tokio::test]
    async fn test_read_file_to_buffer() {
        let path = &Path::new(CARGO_MANIFEST_DIR).join("test_files/file.json");
        let result = read_file_to_buffer(path).await;
        assert!(result.is_ok());
        let buffer = result.unwrap();
        assert!(!buffer.is_empty());
    }

    #[tokio::test]
    async fn test_read_nonexistens_file() {
        let path = Path::new("/123qwe1123.json");
        let result = read_file_to_buffer(path).await;
        assert_io_error(result);
    }

    #[tokio::test]
    async fn test_empty_file() {
        let path = &Path::new(CARGO_MANIFEST_DIR).join("empty_file.py");
        let result = read_file_to_buffer(path).await;
        assert_io_error(result);
    }

    #[tokio::test]
    async fn test_access_denied() {
        let path = &Path::new(CARGO_MANIFEST_DIR).join("test_files/file_access_denied.txt");
        fs::set_permissions(path, Permissions::from_mode(0o200))
            .await
            .unwrap();

        let result = read_file_to_buffer(path).await;
        assert_io_error(result);
    }

    #[tokio::test]
    async fn test_validate_project_id_exist() {
        let testops_api = TestopsApi::default_test();
        let project_id_exist: u32 = 2;
        let res = validate_project_id(project_id_exist, &testops_api).await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_validate_project_id_nonexist() {
        let testops_api = TestopsApi::default_test();
        let project_id_nonexist: u32 = 28888;
        let res = validate_project_id(project_id_nonexist, &testops_api).await;
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
        let zip_dir = zip_directory(&full_path.to_str().unwrap())
            .await
            .unwrap_or_default();
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
        let _ = File::create(full_path.join("some_file.json"))
            .await
            .unwrap();
        // Используем функцию zip_directory и проверяем полученный путь
        let zip_dir = zip_directory(&full_path.to_str().unwrap())
            .await
            .unwrap_or_default();
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
        let testops_api_client = TestopsApi::default_test();
        let launch_info = LaunchInfo::default();
        let path_archive = PathBuf::from(format!(
            "{}/test_files/testops_results_report_1735389182.zip",
            env!("CARGO_MANIFEST_DIR")
        ));
        let _ = testops_api_client
            .post_upload_report(&path_archive, &launch_info)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_scenario_parse() {
        let data = fs::read_to_string(format!(
            "{}/test_files/scenario_with_expected_result.json",
            env!("CARGO_MANIFEST_DIR")
        ))
        .await
        .unwrap();
        let scenario: Scenario = serde_json::from_str(&data)
            .expect("Ошибка парсинга JSON из файла /test_files/scenario_with_expected_result.json");
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
}
