use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};
use walkdir::{DirEntry, WalkDir};
// TODO навести порядок, порефачить код, написать тесты.

/// Пользовательский тип ошибки для большей ясности.
/// В реальном проекте для этого удобно использовать крейт `thiserror`.
#[derive(Debug)]
pub enum ParserError {
    Io(io::Error),
}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParserError::Io(e) => write!(f, "Ошибка ввода/вывода: {}", e),
        }
    }
}

impl Error for ParserError {}

impl From<io::Error> for ParserError {
    fn from(err: io::Error) -> Self {
        ParserError::Io(err)
    }
}

pub type TestMap = HashMap<i32, String>;

/// Состояние парсера, которое передается между итерациями.
struct ParserState<'a> {
    test_map: TestMap,
    current_class_name: Option<String>,
    last_allure_id: Option<i32>,
    file_path_str: &'a str,
}

impl<'a> ParserState<'a> {
    fn new(file_path_str: &'a str) -> Self {
        Self {
            test_map: TestMap::new(),
            current_class_name: None,
            last_allure_id: None,
            file_path_str,
        }
    }
}

pub fn search_test_files(input_path: &Path) -> Result<Vec<PathBuf>, ParserError> {
    Ok(WalkDir::new(input_path)
        .into_iter()
        .filter_map(process_dir_entry)
        .collect())
}

/// Обрабатывает запись директории, фильтруя и проверяя файлы тестов.
fn process_dir_entry(result_dir_entry: Result<DirEntry, walkdir::Error>) -> Option<PathBuf> {
    // .ok()? обрабатывает Result, превращая Err в None и выходя из замыкания.
    let dir_entry = result_dir_entry.ok()?;
    let path = dir_entry.path();

    // Проверяем общие условия для пути и типа файла.
    if !path.is_file() || path.to_string_lossy().contains("__") {
        return None;
    }

    // Безопасно работаем с именем файла, отбрасывая некорректные.
    // `?` здесь отбросит пути, у которых проблемы с именем файла или кодировкой UTF-8.
    let file_name = path.file_name()?.to_str()?;
    if file_name.starts_with("test_") && file_name.ends_with(".py") {
        Some(path.to_path_buf())
    } else {
        None
    }
}

/// Парсит Python-файл и извлекает пути к тестам по их allure.id.
///
/// # Arguments
///
/// * `file_path_str` - Строка с полным путем к файлу.
///
/// # Returns
///
/// * `Ok(TestMap)` - `HashMap`, где ключ - это allure id, а значение - полный путь к тесту.
/// * `Err(ParserError)` - Ошибка, если файл не может быть прочитан или обработан.
pub fn parse_tests_from_file(file_path: &Path) -> Result<TestMap, ParserError> {
    let file_path_str = file_path
        .to_str()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Invalid UTF-8 in file path"))?;

    let file = File::open(file_path)?;
    let reader = io::BufReader::new(file);

    let final_state =
        reader
            .lines()
            .try_fold(ParserState::new(file_path_str), |mut state, line_result| {
                let line = line_result?;
                let trimmed_line = line.trim();

                if trimmed_line.starts_with("class Test") {
                    state.current_class_name = parse_class_name(trimmed_line);
                } else if trimmed_line.contains("@allure.id") {
                    if let Some(id) = parse_allure_id(trimmed_line) {
                        state.last_allure_id = Some(id);
                    }
                } else if trimmed_line.starts_with("def test_") {
                    if let (Some(id), Some(test_name)) =
                        (state.last_allure_id.take(), parse_test_name(trimmed_line))
                    {
                        let full_path = match &state.current_class_name {
                            Some(class_name) => {
                                format!("{}::{}::{}", state.file_path_str, class_name, test_name)
                            }
                            None => format!("{}::{}", state.file_path_str, test_name),
                        };
                        state.test_map.insert(id, full_path);
                    }
                }
                Ok::<_, ParserError>(state)
            })?;

    Ok(final_state.test_map)
}

/// Извлекает имя класса из строки вида `class TestMyClass:`.
fn parse_class_name(line: &str) -> Option<String> {
    line.split_whitespace()
        .nth(1)?
        .strip_suffix(':')
        .map(|s| s.to_string())
}

/// Извлекает ID из строки вида `@allure.id("7532")` или `@allure.id('7532')`.
fn parse_allure_id(line: &str) -> Option<i32> {
    line.split(|c| ['"', '\''].contains(&c))
        .nth(1)?
        .parse()
        .ok()
}

/// Извлекает имя теста из строки вида `def test_bad(self):`.
/// Возвращает срез `&str`, избегая аллокации `String`.
fn parse_test_name(line: &str) -> Option<&str> {
    // 1. Находим позицию "def " для определения начала функции.
    line.find("def ")
        // 2. Берём срез строки после "def ".
        .map(|def_pos| &line[def_pos + 4..])
        // 3. Ищем начало имени теста "test_".
        .and_then(|after_def| {
            after_def
                .find("test_")
                .map(|test_pos| &after_def[test_pos..])
        })
        // 4. Находим конец имени теста по символу '('.
        .and_then(|from_test| from_test.find('(').map(|paren_pos| &from_test[..paren_pos]))
        // 5. Убираем лишние пробелы в конце имени.
        .map(|test_name| test_name.trim_end())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    fn test_process_dir_entry() {
        // TODO надо будеть подправить тесты чтобы убрать дублирующиеся код
        let temp_dir = tempfile::tempdir().unwrap();
        let base_path = temp_dir.path();

        // Создаем файлы для разных тестовых случаев
        let valid_test_file = base_path.join("test_example.py");
        std::fs::File::create(&valid_test_file).unwrap();

        // Собираем результаты работы функции для всех записей в директории
        let results: Vec<PathBuf> = WalkDir::new(base_path)
            .into_iter()
            .map(process_dir_entry)
            .flatten()
            .collect();

        // Проверяем, что в результате только один валидный тестовый файл
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], valid_test_file);
    }

    #[rstest]
    #[case("module.py")]
    #[case("tests_module.py")]
    #[case("module.pyc")]
    #[case("__init__.py")]
    fn test_process_dir_entry_invalid_file_name(#[case] file_name: &str) {
        let temp_dir = tempfile::tempdir().unwrap();
        let base_path = temp_dir.path();

        let file = base_path.join(file_name);
        std::fs::File::create(&file).unwrap();
        // Собираем результаты работы функции для всех записей в директории
        let results: Vec<PathBuf> = WalkDir::new(base_path)
            .into_iter()
            .map(process_dir_entry) // Применяем нашу функцию
            .flatten() // Убираем None, оставляя только Some(PathBuf)
            .collect();
        assert_eq!(results.len(), 0);
    }

    #[rstest]
    #[case("subdir")]
    #[case("__pacache__/test_example.py")]
    fn test_process_dir_entry_dir(#[case] file_name: &str) {
        let temp_dir = tempfile::tempdir().unwrap();
        let base_path = temp_dir.path();

        let file = base_path.join(file_name);
        std::fs::create_dir_all(&file).unwrap();
        // Собираем результаты работы функции для всех записей в директории
        let results: Vec<PathBuf> = WalkDir::new(base_path)
            .into_iter()
            .map(process_dir_entry) // Применяем нашу функцию
            .flatten() // Убираем None, оставляя только Some(PathBuf)
            .collect();
        assert_eq!(results.len(), 0);
    }

    #[rstest]
    #[case("class TestMyClass:", Some("TestMyClass".to_string()))]
    #[case("class Another_Test_Class:", Some("Another_Test_Class".to_string()))]
    #[case("class TestMyClass", None)]
    #[case("not a class line", None)]
    #[case("class", None)]
    #[case("class :", Some("".to_string()))]
    fn test_parse_class_name(#[case] line: &str, #[case] expected: Option<String>) {
        assert_eq!(parse_class_name(line), expected);
    }

    #[rstest]
    #[case("@allure.id(\"12345\")", Some(12345))]
    #[case("@allure.id(\"99\")", Some(99))]
    #[case("@allure.id('999')", Some(999))]
    #[case("@allure.id(\"-5\")", Some(-5))]
    #[case("@allure.id(\"not-a-number\")", None)]
    #[case("not an allure id line", None)]
    #[case("@allure.id()", None)]
    #[case("@allure.id(\"\")", None)]
    fn test_parse_allure_id(#[case] line: &str, #[case] expected: Option<i32>) {
        assert_eq!(parse_allure_id(line), expected);
    }

    #[rstest]
    #[case("def test_my_function(self):", Some("test_my_function"))]
    #[case("def test_another(self, fixture1, fixture2):", Some("test_another"))]
    #[case("def test_no_params():", Some("test_no_params"))]
    #[case("def test_no_parens", None)] // Невалидный синтаксис, нет '('.
    #[case("not a function def", None)]
    #[case("def my_func()", None)] // Не является тестом.
    #[case("def   test_with_spaces ( self ) :", Some("test_with_spaces"))]
    fn test_parse_test_name(#[case] line: &str, #[case] expected: Option<&str>) {
        assert_eq!(parse_test_name(line), expected);
    }
}
