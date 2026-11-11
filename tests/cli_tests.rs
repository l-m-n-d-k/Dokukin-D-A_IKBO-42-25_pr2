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
fn valid_config_prints_parameters_and_succeeds() {
    let output = run_app(&["tests/fixtures/valid.yaml"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success());
    assert!(stdout.contains("package_name = express"));
    assert!(stdout.contains("output_file = graph.png"));
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
fn help_is_printed() {
    let output = run_app(&["--help"]);
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("Использование"));
}
