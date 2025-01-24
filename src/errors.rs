use std::fmt;

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
    ParseHeaderValue,
    NotReadFile(String, String),
    ExtensionZip(String),
    NotFileName(String),
    CouldReadLine,
    CantCreateConfig,
}
// Не смогли прочитать файл по пути: {:?}. Получили ошибку: {:?}
impl fmt::Display for WotError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WotError::NotFoundUserDir => write!(f, "Не удалось получить каталоги пользователя"),
            WotError::InvalidURL => write!(f, "Введенная строка должна быть URL"),
            WotError::NotReadFileByPath(file_path) => {
                write!(f, "Получили ошибку при чтение файла по пути: {file_path}")
            }
            WotError::InvalidToken => {
                write!(f, "Ваш токен не прошел валидацию, попробуйте еще раз")
            }
            WotError::ProjectIdNotFound(project_id) => {
                write!(f, "Project with ID == {project_id} not found")
            }
            WotError::NotParseConfig => write!(f, "Не смогли распарсить конфиг"),
            WotError::NotFoundDirByPath(file_path) => {
                write!(f, "Не нашли директорию по пути: <{file_path}>")
            }
            WotError::ParseFileNameToStr => write!(f, "Не смогли привести имя файла к строке"),
            WotError::ParseHeaderValue => write!(f, "Не смогли преобразовать HeaderValue"),
            WotError::NotReadFile(path, error) => write!(
                f,
                "Не смогли прочитать файл по пути: \"{path}\". Получили ошибку: {error}",
            ),
            WotError::ExtensionZip(extension_file) => write!(
                f,
                "Нужен файл с расширением .zip, был передан файл: *.{extension_file}"
            ),
            WotError::NotFileName(file_path) => {
                write!(f, "Не удалось получить имя файла из пути: \"{file_path}\"")
            }
            WotError::ProjectIdMoreThenZero => {
                write!(f, "project_id должен быть целым числом > 0")
            }
            WotError::CouldReadLine => {
                write!(f, "Couldn't read the line")
            }
            WotError::CantCreateConfig => {
                write!(f, "Не смогли создать конфиг")
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
                "Не смогли преобразовать файл {file_name} для API-запроса. Ошибка: {error}"
            ),
            WotApiError::EmptyResponse(method) => {
                write!(f, "Получили пустой ответ от метода {method}")
            }
            WotApiError::ParsingResponse(method, error) => {
                write!(
                    f,
                    "При парсинге ответа метода {method} получили ошибку: {error}"
                )
            }
        }
    }
}

impl std::error::Error for WotApiError {}
