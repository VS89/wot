use directories::UserDirs;
use std::error::Error;
use std::fs::{self, read_dir, File};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use zip::{write::SimpleFileOptions, CompressionMethod, ZipWriter};

/// Получаем директорию для архива
fn get_dir_archive() -> Result<PathBuf, Box<dyn Error>> {
    if let Some(user_dirs) = UserDirs::new() {
        if let Some(desktop_dir) = user_dirs.desktop_dir() {
            Ok(desktop_dir.join(format!(
                "testops_results_report_{}.zip",
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
            )))
        } else {
            return Err("Не смогли найти путь до рабочего стола".into());
        }
    } else {
        return Err("Не удалось получить каталоги пользователя".into());
    }
}

/// Архивация директории в *.zip
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
        _ => return Err(format!("Не нашли директорию по пути: <{:?}>", path_to_report_dir).into()),
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
                        None => return Err("Не смогли привести имя файла к строке".into()),
                    };
                    zip.start_file(format!("{}", file_name_archive).to_string(), options)?;
                    zip.write_all(&buffer)?;
                }
            }
            Err(e) => return Err(format!("Получили ошибку: {}", e).into()),
        };
    }
    // Завершаем запись архива
    zip.finish()?;
    Ok(dir_archive)
}

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;

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
        let expected_error =
            "Не нашли директорию по пути: <\"nonexistent_dir_for_test\">".to_string();
        let actual_error = zip_directory("nonexistent_dir_for_test");
        assert_eq!(
            expected_error,
            actual_error.unwrap_err().to_string(),
            "Ошибка, когда передаем несуществующую директорию"
        );
    }
}
