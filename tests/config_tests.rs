//! Тесты чтения конфигурационного файла и разбора аргументов.

use std::path::{Path, PathBuf};

use depgraph::cli::{Command, DEFAULT_CONFIG_PATH, parse_args};
use depgraph::config::{Config, load_config};
use depgraph::error::AppError;

/// Загружает файл из каталога `tests/fixtures`.
fn load_fixture(name: &str) -> Result<Config, AppError> {
    load_config(&Path::new("tests/fixtures").join(name))
}

/// Превращает список строк в аргументы командной строки.
fn args(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| s.to_string()).collect()
}

#[test]
fn valid_config_is_loaded() {
    let config = load_fixture("valid.yaml").expect("конфигурация корректна");
    assert_eq!(config.package_name, "express");
    assert_eq!(config.repository, "https://registry.npmjs.org");
    assert!(!config.test_mode);
    assert_eq!(config.version, "4.18.2");
    assert_eq!(config.output_file, "graph.png");
}

#[test]
fn valid_test_mode_config_is_loaded() {
    let config = load_fixture("valid_test_mode.yaml").expect("конфигурация корректна");
    assert!(config.test_mode);
    assert_eq!(config.package_name, "A");
}

#[test]
fn config_is_printed_as_key_value() {
    let config = load_fixture("valid.yaml").unwrap();
    let expected = "package_name = express\n\
                    repository = https://registry.npmjs.org\n\
                    test_mode = false\n\
                    version = 4.18.2\n\
                    output_file = graph.png\n";
    assert_eq!(config.to_string(), expected);
}

#[test]
fn missing_file_is_read_error() {
    assert!(matches!(load_fixture("no_such.yaml"), Err(AppError::ReadConfig { .. })));
}

#[test]
fn structural_problems_are_parse_errors() {
    for name in [
        "missing_field.yaml",
        "unknown_field.yaml",
        "wrong_type.yaml",
        "invalid_yaml.yaml",
        "empty.yaml",
    ] {
        let result = load_fixture(name);
        assert!(matches!(result, Err(AppError::ParseConfig { .. })), "{name}: {result:?}");
    }
}

#[test]
fn all_bad_values_are_reported_at_once() {
    for name in ["bad_values.yaml", "bad_test_mode.yaml"] {
        let Err(AppError::InvalidConfig(errors)) = load_fixture(name) else {
            panic!("{name}: ожидалась ошибка InvalidConfig");
        };
        let fields: Vec<_> = errors.iter().map(|e| e.field).collect();
        assert_eq!(fields, ["package_name", "repository", "version", "output_file"], "{name}");
    }
}

#[test]
fn args_without_path_use_default_config() {
    let command = parse_args(args(&[])).unwrap();
    assert_eq!(command, Command::Run(PathBuf::from(DEFAULT_CONFIG_PATH)));
}

#[test]
fn args_with_path_and_help() {
    assert_eq!(parse_args(args(&["my.yaml"])).unwrap(), Command::Run("my.yaml".into()));
    assert_eq!(parse_args(args(&["--help"])).unwrap(), Command::Help);
    assert_eq!(parse_args(args(&["-h"])).unwrap(), Command::Help);
}

#[test]
fn bad_args_are_usage_errors() {
    assert!(matches!(parse_args(args(&["a.yaml", "b.yaml"])), Err(AppError::Usage(_))));
    assert!(matches!(parse_args(args(&["--verbose"])), Err(AppError::Usage(_))));
}
