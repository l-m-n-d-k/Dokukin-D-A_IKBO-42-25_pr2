//! Типы ошибок приложения.
//!
//! Все ошибки собраны в одном перечислении [`AppError`], чтобы `main`
//! мог обработать их в одном месте и вывести понятное сообщение.

use std::fmt;
use std::io;
use std::path::PathBuf;

use crate::cli::USAGE;

/// Ошибка в значении одного параметра конфигурации.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldError {
    /// Имя параметра так, как оно записано в YAML-файле.
    pub field: &'static str,
    /// Описание проблемы для пользователя.
    pub message: String,
}

impl FieldError {
    /// Создаёт ошибку для параметра `field` с описанием `message`.
    pub fn new(field: &'static str, message: impl Into<String>) -> Self {
        Self {
            field,
            message: message.into(),
        }
    }
}

impl fmt::Display for FieldError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.field, self.message)
    }
}

/// Все ошибки, которые может вернуть приложение.
#[derive(Debug)]
pub enum AppError {
    /// Неверные аргументы командной строки.
    Usage(String),
    /// Конфигурационный файл не удалось прочитать (нет файла, нет прав и т.п.).
    ReadConfig {
        /// Путь к файлу, который пытались прочитать.
        path: PathBuf,
        /// Исходная ошибка ввода-вывода.
        source: io::Error,
    },
    /// Файл не является корректным YAML, в нём не хватает параметров,
    /// есть лишние параметры или у значения неверный тип.
    ParseConfig {
        /// Путь к разбираемому файлу.
        path: PathBuf,
        /// Исходная ошибка разбора YAML.
        source: serde_yaml::Error,
    },
    /// Один или несколько параметров имеют недопустимые значения.
    InvalidConfig(Vec<FieldError>),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Usage(message) => write!(f, "{message}\n\n{USAGE}"),
            Self::ReadConfig { path, source } => {
                write!(f, "не удалось прочитать файл '{}': {source}", path.display())
            }
            Self::ParseConfig { path, source } => {
                write!(f, "некорректный конфигурационный файл '{}': {source}", path.display())
            }
            Self::InvalidConfig(errors) => {
                write!(f, "недопустимые значения параметров:")?;
                for error in errors {
                    write!(f, "\n  - {error}")?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for AppError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ReadConfig { source, .. } => Some(source),
            Self::ParseConfig { source, .. } => Some(source),
            Self::Usage(_) | Self::InvalidConfig(_) => None,
        }
    }
}
