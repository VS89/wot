pub mod cli_app;
pub mod command_logic;
pub mod config;
pub mod constants;
pub mod errors;
pub mod external_api;
pub mod utils;

use config::Config;
use constants::CONFIG_DIR;
use directories::UserDirs;
use errors::WotError;

use external_api::base_api_client::ApiError;
use external_api::testops::TestopsApiClient;
use std::collections::HashSet;
use std::error::Error;
use std::fs::{self, read_dir, File};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use zip::{write::SimpleFileOptions, CompressionMethod, ZipWriter};

/// Get path directory with report tests
fn get_dir_archive() -> Result<PathBuf, WotError> {
    if let Some(user_dirs) = UserDirs::new() {
        let archive_name = format!(
            "testops_results_report_{}.zip",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
        );
        Ok(user_dirs.home_dir().join(CONFIG_DIR).join(archive_name))
    } else {
        Err(WotError::NotFoundUserDir)
    }
}

/// Validate project id. Check project_id is in project ids list testops
async fn validate_project_id(
    project_id: &u32,
    config: &Config,
) -> Result<bool, Box<dyn std::error::Error>> {
    let testops = TestopsApiClient::new(config);
    let set_project_ids: HashSet<u32> = testops.get_all_project_ids().await?;
    match set_project_ids.contains(project_id) {
        true => Ok(true),
        false => Err(WotError::ProjectIdNotFound(*project_id).into()),
    }
}

/// Directory archive to *.zip
pub fn zip_directory(path_to_report_dir: &str) -> Result<PathBuf, Box<dyn Error>> {
    let dir_archive = get_dir_archive()?;
    if let Some(parent) = &dir_archive.parent() {
        fs::create_dir_all(parent)?;
    }
    let zip_file = File::create(&dir_archive)?;
    let mut zip = ZipWriter::new(zip_file);

    // Настройки для файла в архиве
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .unix_permissions(0o755);
    let read_directory = match read_dir(path_to_report_dir) {
        Ok(value) => value,
        _ => return Err(WotError::NotFoundDirByPath(path_to_report_dir.to_string()).into()),
    };
    for entry in read_directory {
        match entry {
            Ok(value) => {
                // Читаем содержимое исходного файла
                if value.file_type()?.is_file() {
                    let mut file = File::open(value.path())?;
                    let mut buffer = Vec::new();
                    file.read_to_end(&mut buffer)?;
                    // Добавляем файл в ZIP-архив
                    let file_name_archive = match value.file_name().to_str() {
                        Some(file_name) => file_name.to_string(),
                        None => return Err(WotError::ParseFileNameToStr.into()),
                    };
                    zip.start_file(file_name_archive.to_string(), options)?;
                    zip.write_all(&buffer)?;
                }
            }
            Err(e) => return Err(e.into()),
        };
    }
    // Завершаем запись архива
    zip.finish()?;
    Ok(dir_archive)
}

/// Create file in current directory
///
/// Return full path to created file
fn create_file_in_current_directory(file_name: &str, content: &[u8]) -> Result<String, WotError> {
    let mut file = match File::create(file_name) {
        Ok(value) => value,
        Err(_) => return Err(WotError::CouldNotCreateFile),
    };
    let _ = file.write_all(content);
    let mut path = match std::env::current_dir() {
        Ok(value) => value,
        Err(_) => return Err(WotError::CouldNotCreateFile)
    };
    path.push(file_name);
    Ok(path.display().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;
    use std::path::Path;

    #[tokio::test]
    async fn test_validate_project_id_exist() {
        let path: &Path = Path::new("/Users/valentins/.config/wot/test_config.json");
        let config = Config::get_config(path.to_path_buf()).unwrap();
        let project_id_exist: u32 = 2;
        let res = validate_project_id(&project_id_exist, &config)
            .await
            .unwrap();
        assert!(res, "Не нашли проект с id == {}", project_id_exist);
    }

    #[tokio::test]
    async fn test_validate_project_id_nonexist() {
        let path: &Path = Path::new("/Users/valentins/.config/wot/test_config.json");
        let config = Config::get_config(path.to_path_buf()).unwrap();
        let project_id_nonexist: u32 = 28888;
        let res = validate_project_id(&project_id_nonexist, &config)
            .await
            .unwrap_err()
            .to_string();
        assert_eq!(
            res,
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

    #[test]
    /// Проверяем архивацию пустой папки
    fn test_zip_empty_directory() {
        // Получаем полный путь до папки для архивирования и создаем ее(пустая без файлов)
        let binding = UserDirs::new().unwrap();
        let desktop_dir = binding.desktop_dir().unwrap();
        let full_path: PathBuf = desktop_dir.join("dir_for_test_zip_directory");
        let _ = fs::create_dir_all(&full_path);
        // Используем функцию zip_directory и проверяем полученный путь
        let zip_dir = zip_directory(&full_path.to_str().unwrap()).unwrap_or_default();
        let re = Regex::new(r"testops_results_report_\d+\.zip").unwrap();
        if !re.is_match(&zip_dir.to_str().unwrap_or_default()) {
            assert!(
                false,
                "Получили dir_path == <{:?}>. Ожидали что имя файла будет соответствовать шаблону testops_results_report_\\d+\\.zip",
                &zip_dir.to_str().unwrap_or_default()
            );
        }
    }

    #[test]
    /// Проверяем архивацию директории, в которой есть файл
    fn test_zip_dir_with_one_file() {
        // Получаем полный путь до папки для архивирования и создаем ее(пустая без файлов)
        let binding = UserDirs::new().unwrap();
        let desktop_dir = binding.desktop_dir().unwrap();
        let full_path: PathBuf = desktop_dir.join("dir_for_test_zip_directory_with_one_file");
        let _ = fs::create_dir_all(&full_path);
        // Создаем файл в директории
        let _ = File::create(full_path.join("some_file.json")).unwrap();
        // Используем функцию zip_directory и проверяем полученный путь
        let zip_dir = zip_directory(&full_path.to_str().unwrap()).unwrap_or_default();
        let re = Regex::new(r"testops_results_report_\d+\.zip").unwrap();
        if !re.is_match(&zip_dir.to_str().unwrap_or_default()) {
            assert!(
                false,
                "Получили dir_path == <{:?}>. Ожидали что имя файла будет соответствовать шаблону testops_results_report_\\d+\\.zip",
                &zip_dir.to_str().unwrap_or_default()
            );
        }
    }

    #[test]
    /// Проверяем архивацию несуществующей директории
    fn test_nonexistent() {
        let expected_error = "Could not find the directory at path: <nonexistent_dir_for_test>";
        let actual_error = zip_directory("nonexistent_dir_for_test");
        assert_eq!(
            expected_error,
            actual_error.unwrap_err().to_string(),
            "Ошибка, когда передаем несуществующую директорию"
        );
    }
}
