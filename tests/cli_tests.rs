//! Тесты запуска собранного приложения целиком.

use std::process::{Command, Output};

/// Запускает приложение `depgraph` с указанными аргументами.
fn run_app(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_depgraph"))
        .args(args)
        .output()
        .expect("не удалось запустить приложение")
}

/// Стандартный вывод приложения в виде строки.
fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

/// Вывод ошибок приложения в виде строки.
fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[test]
fn invalid_config_prints_error_and_fails() {
    let output = run_app(&["tests/fixtures/bad_values.yaml"]);
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(stderr(&output).starts_with("Ошибка:"));
    assert!(stderr(&output).contains("version"));
}

#[test]
fn test_mode_prints_the_graph() {
    let output = run_app(&["tests/fixtures/test_diamond.yaml"]);
    assert!(output.status.success());
    let expected = "Граф зависимостей A: пакетов 4, зависимостей 4\n\
                    \x20 A -> B, C\n\
                    \x20 B -> D\n\
                    \x20 C -> D\n\
                    \x20 D -> (нет зависимостей)\n";
    assert_eq!(stdout(&output), expected);
}

#[test]
fn test_mode_handles_cycles() {
    let output = run_app(&["tests/fixtures/test_cycle.yaml"]);
    assert!(output.status.success());
    assert!(stdout(&output).contains("C -> A"));
}

#[test]
fn test_mode_reports_unknown_root_package() {
    let output = run_app(&["tests/fixtures/test_missing_root.yaml"]);
    assert!(!output.status.success());
    assert!(stderr(&output).contains("'Q'"), "{}", stderr(&output));
}

#[test]
fn test_mode_reports_broken_repository_file() {
    let output = run_app(&["tests/fixtures/test_bad_syntax.yaml"]);
    assert!(!output.status.success());
    assert!(stderr(&output).contains("строка 2"), "{}", stderr(&output));
}

#[test]
fn unreachable_repository_prints_error_and_fails() {
    let output = run_app(&["tests/fixtures/unreachable.yaml"]);
    assert!(!output.status.success());
    assert!(stderr(&output).contains("не удалось подключиться"), "{}", stderr(&output));
}

#[test]
fn help_is_printed() {
    let output = run_app(&["--help"]);
    assert!(output.status.success());
    assert!(stdout(&output).contains("Использование"));
}
