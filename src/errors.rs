use std::fmt;

pub const FAILED_WRITE_STDOUT: &str = "Failed to write to stdout";
pub const FAILED_FLUSH_STDOUT: &str = "Failed to flush stdout";
pub const PARSE_HEADER_VALUE: &str = "Could not convert HeaderValue";
pub const COULD_READ_LINE: &str = "Couldn't read the line";
pub const CANT_CREATE_CONFIG: &str = "Couldn't create a config";

#[derive(Debug)]
pub enum WotError {
    NotFoundUserDir,
    InvalidURL,
    ProjectIdMoreThenZero,
    NotReadFileByPath(String),
    InvalidToken,
    ProjectIdNotFound(u32),
    NotParseConfig,
    NotFoundDirByPath(String),
    ParseFileNameToStr,
    NotReadFile(String, String),
    ExtensionZip(String),
    NotFileName(String),
}
// Не смогли прочитать файл по пути: {:?}. Получили ошибку: {:?}
impl fmt::Display for WotError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WotError::NotFoundUserDir => write!(f, "Failed to retrieve the user's directories"),
            WotError::InvalidURL => write!(f, "The string entered must be a URL"),
            WotError::NotReadFileByPath(file_path) => {
                write!(
                    f,
                    "Received an error when reading a file at path: {file_path}"
                )
            }
            WotError::InvalidToken => {
                write!(f, "Your token failed validation, please try again")
            }
            WotError::ProjectIdNotFound(project_id) => {
                write!(f, "Project with ID == {project_id} not found")
            }
            WotError::NotParseConfig => write!(f, "Couldn't parse the config"),
            WotError::NotFoundDirByPath(file_path) => {
                write!(f, "Could not find the directory at path: <{file_path}>")
            }
            WotError::ParseFileNameToStr => write!(f, "Could not cast the filename to a string"),
            WotError::NotReadFile(path, error) => write!(
                f,
                "Couldn't read the file in the path: \"{path}\". We got an error: {error}",
            ),
            WotError::ExtensionZip(extension_file) => write!(
                f,
                "Need a file with a .zip extension, a file was transferred: *.{extension_file}"
            ),
            WotError::NotFileName(file_path) => {
                write!(
                    f,
                    "Failed to get the file name from the path:\"{file_path}\""
                )
            }
            WotError::ProjectIdMoreThenZero => {
                write!(f, "project_id must be an integer > 0")
            }
        }
    }
}

impl std::error::Error for WotError {}

#[derive(Debug)]
pub enum WotApiError {
    Multipart(String, String),
    EmptyResponse(String),
    ParsingResponse(String, String),
}

impl fmt::Display for WotApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WotApiError::Multipart(file_name, error) => write!(
                f,
                "Could not convert file {file_name} for API request. Error: {error}"
            ),
            WotApiError::EmptyResponse(method) => {
                write!(f, "Received an empty response from method {method}")
            }
            WotApiError::ParsingResponse(method, error) => {
                write!(
                    f,
                    "When parsing the response of method {method}, we got an error: {error}"
                )
            }
        }
    }
}

impl std::error::Error for WotApiError {}
