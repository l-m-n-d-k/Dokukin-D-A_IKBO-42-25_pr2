//! Конфигурация приложения: структура параметров, чтение из YAML и вывод.

use std::fmt;
use std::fs;
use std::path::Path;

use serde::Deserialize;

use crate::error::AppError;
use crate::validate;

/// Параметры, которые пользователь задаёт в конфигурационном файле.
///
/// Все поля обязательны. Параметры, которых нет в этой структуре,
/// считаются ошибкой (`deny_unknown_fields`), чтобы опечатка в имени
/// параметра не проходила незамеченной.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Имя анализируемого пакета.
    pub package_name: String,
    /// URL-адрес репозитория или путь к файлу тестового репозитория.
    pub repository: String,
    /// Режим работы с тестовым репозиторием.
    pub test_mode: bool,
    /// Версия пакета.
    pub version: String,
    /// Имя сгенерированного файла с изображением графа.
    pub output_file: String,
}

impl Config {
    /// Возвращает все параметры в виде пар «ключ — значение»
    /// в том порядке, в котором они описаны в структуре.
    pub fn entries(&self) -> [(&'static str, String); 5] {
        [
            ("package_name", self.package_name.clone()),
            ("repository", self.repository.clone()),
            ("test_mode", self.test_mode.to_string()),
            ("version", self.version.clone()),
            ("output_file", self.output_file.clone()),
        ]
    }
}

impl fmt::Display for Config {
    /// Выводит каждый параметр на отдельной строке в формате `ключ = значение`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (key, value) in self.entries() {
            writeln!(f, "{key} = {value}")?;
        }
        Ok(())
    }
}

/// Разбирает YAML-текст в [`Config`] без проверки значений.
///
/// # Ошибки
///
/// Возвращает ошибку `serde_yaml`, если текст не является корректным YAML,
/// не хватает параметров, есть лишние параметры или у значения неверный тип.
pub fn parse_yaml(text: &str) -> Result<Config, serde_yaml::Error> {
    serde_yaml::from_str(text)
}

/// Читает конфигурационный файл, разбирает его и проверяет все параметры.
///
/// Относительные пути внутри файла (`repository` в тестовом режиме,
/// `output_file`) считаются от текущего рабочего каталога.
///
/// # Ошибки
///
/// - [`AppError::ReadConfig`] — файл не удалось прочитать;
/// - [`AppError::ParseConfig`] — файл не является корректной конфигурацией;
/// - [`AppError::InvalidConfig`] — у параметров недопустимые значения.
pub fn load_config(path: &Path) -> Result<Config, AppError> {
    let text = fs::read_to_string(path).map_err(|source| AppError::ReadConfig {
        path: path.to_path_buf(),
        source,
    })?;

    let config = parse_yaml(&text).map_err(|source| AppError::ParseConfig {
        path: path.to_path_buf(),
        source,
    })?;

    validate::validate_config(&config).map_err(AppError::InvalidConfig)?;
    Ok(config)
}
