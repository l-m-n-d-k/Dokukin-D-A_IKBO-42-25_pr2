//! Тесты запуска собранного приложения целиком.

use std::process::{Command, Output};

/// Запускает приложение `depgraph` с указанными аргументами.
fn run_app(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_depgraph"))
        .args(args)
        .output()
        .expect("не удалось запустить приложение")
}

#[test]
fn invalid_config_prints_error_and_fails() {
    let output = run_app(&["tests/fixtures/bad_values.yaml"]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(stderr.starts_with("Ошибка:"));
    assert!(stderr.contains("version"));
}

#[test]
fn test_mode_reports_that_it_is_not_ready() {
    let output = run_app(&["tests/fixtures/valid_test_mode.yaml"]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success());
    assert!(stderr.contains("этапе 3"), "{stderr}");
}

#[test]
fn unreachable_repository_prints_error_and_fails() {
    let output = run_app(&["tests/fixtures/unreachable.yaml"]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success());
    assert!(stderr.contains("не удалось подключиться"), "{stderr}");
}

#[test]
fn help_is_printed() {
    let output = run_app(&["--help"]);
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("Использование"));
}
