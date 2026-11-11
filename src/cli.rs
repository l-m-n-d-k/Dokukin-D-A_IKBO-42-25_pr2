//! Разбор аргументов командной строки.
//!
//! Приложение принимает не более одного аргумента — путь к
//! конфигурационному файлу. Все остальные настройки задаются в YAML.

use std::path::PathBuf;

use crate::error::AppError;

/// Путь к конфигурационному файлу, если он не указан явно.
pub const DEFAULT_CONFIG_PATH: &str = "config.yaml";

/// Справка, которая выводится по `--help` и при ошибке в аргументах.
pub const USAGE: &str = "\
Использование: depgraph [ПУТЬ_К_КОНФИГУ]

Инструмент визуализации графа зависимостей npm-пакетов.
Все параметры задаются в конфигурационном файле формата YAML.

Аргументы:
  ПУТЬ_К_КОНФИГУ  путь к YAML-файлу (по умолчанию: config.yaml)
  -h, --help      показать эту справку";

/// Что нужно сделать приложению после разбора аргументов.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// Показать справку и выйти.
    Help,
    /// Запустить приложение с конфигурацией из указанного файла.
    Run(PathBuf),
}

/// Разбирает аргументы командной строки (без имени программы).
///
/// # Ошибки
///
/// Возвращает [`AppError::Usage`], если передано больше одного аргумента
/// или указан неизвестный флаг.
pub fn parse_args<I>(args: I) -> Result<Command, AppError>
where
    I: IntoIterator<Item = String>,
{
    let mut args = args.into_iter();
    let first = args.next();

    if let Some(extra) = args.next() {
        return Err(AppError::Usage(format!("лишний аргумент: '{extra}'")));
    }

    match first.as_deref() {
        None => Ok(Command::Run(PathBuf::from(DEFAULT_CONFIG_PATH))),
        Some("-h" | "--help") => Ok(Command::Help),
        Some(flag) if flag.starts_with('-') => {
            Err(AppError::Usage(format!("неизвестный флаг: '{flag}'")))
        }
        Some(path) => Ok(Command::Run(PathBuf::from(path))),
    }
}
