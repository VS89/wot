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

    for entry in read_dir(path_to_report_dir)? {
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
                        None => return Err("Какая-то херня".into()),
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
}
