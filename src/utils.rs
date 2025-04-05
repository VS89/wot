use tokio::{fs::File, io::AsyncReadExt};
use std::path::Path;
use zip::ZipArchive;
use std::io::Cursor;

use crate::external_api::base_api_client::ApiError;

pub async fn read_file_to_buffer(path: &Path) -> Result<Vec<u8>, ApiError> {
    let mut file = File::open(path).await?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).await?;
    Ok(buffer)
}

/// Get file name with extension
pub fn get_file_name(path: &Path) -> Result<String, ApiError> {
    if path.is_dir() {
        return Err(ApiError::InvalidFileName)
    }
    Ok(path.file_name()
        .and_then(|name| name.to_str())
        .ok_or(ApiError::InvalidFileName)?
        .to_string())
}

pub fn validate_zip_archive(buffer: &Vec<u8>) -> Result<(), ApiError> {
    let cursor = Cursor::new(buffer);
    if ZipArchive::new(cursor).is_err() {
        return Err(ApiError::InvalidFileFormat)
    }
    Ok(())
}

#[cfg(test)]
mod tests {

    use super::*;
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
        fs::set_permissions(path, Permissions::from_mode(0o200)).await.unwrap();

        let result = read_file_to_buffer(path).await;
        assert_io_error(result);
    }
}