//! Тесты запуска собранного приложения целиком.

mod common;

use std::fs;
use std::process::{Command, Output};

use common::{fake_png, serve_once, temp_path};
use depgraph::render::RENDER_URL_ENV;

/// Запускает приложение `depgraph` с указанными аргументами.
fn run_app(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_depgraph"))
        .args(args)
        .output()
        .expect("не удалось запустить приложение")
}

/// Запускает приложение с конфигом `fixture`, у которого выходной файл заменён
/// на временный, а сервис отрисовки — на локальный сервер с ответом
/// `status` и телом `body` (код статуса входит в имена временных файлов, чтобы
/// параллельные тесты не мешали друг другу). Возвращает результат и путь к файлу.
fn run_with_renderer(fixture: &str, status: &str, body: Vec<u8>) -> (Output, std::path::PathBuf) {
    let tag = status.split_whitespace().next().unwrap_or_default();
    let output_file = temp_path(&format!("{fixture}-{tag}.png"));
    let text = fs::read_to_string(format!("tests/fixtures/{fixture}.yaml")).unwrap();
    let config = text.replace("graph.png", output_file.to_str().unwrap());
    let config_file = temp_path(&format!("{fixture}-{tag}.yaml"));
    fs::write(&config_file, config).unwrap();

    let (address, _server) = serve_once(status, body);
    let output = Command::new(env!("CARGO_BIN_EXE_depgraph"))
        .arg(&config_file)
        .env(RENDER_URL_ENV, address)
        .output()
        .expect("не удалось запустить приложение");
    fs::remove_file(config_file).unwrap();
    (output, output_file)
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
fn test_mode_prints_mermaid_and_saves_png() {
    let (output, png) = run_with_renderer("test_diamond", "200 OK", fake_png());
    assert!(output.status.success(), "{}", stderr(&output));
    let expected = "graph TD\n\
                    \x20   n0[\"A\"]\n\
                    \x20   n1[\"B\"]\n\
                    \x20   n2[\"C\"]\n\
                    \x20   n3[\"D\"]\n\
                    \x20   n0 --> n1\n\
                    \x20   n0 --> n2\n\
                    \x20   n1 --> n3\n\
                    \x20   n2 --> n3\n";
    assert!(stdout(&output).starts_with(expected), "{}", stdout(&output));
    assert_eq!(fs::read(&png).unwrap(), fake_png());
    fs::remove_file(png).unwrap();
}

#[test]
fn test_mode_handles_cycles() {
    let (output, png) = run_with_renderer("test_cycle", "200 OK", fake_png());
    assert!(output.status.success(), "{}", stderr(&output));
    assert!(stdout(&output).contains("n2 --> n0"));
    fs::remove_file(png).unwrap();
}

#[test]
fn render_failure_keeps_mermaid_text_and_fails() {
    let (output, png) = run_with_renderer("test_diamond", "503 Service Unavailable", Vec::new());
    assert!(!output.status.success());
    assert!(stdout(&output).starts_with("graph TD"), "{}", stdout(&output));
    assert!(stderr(&output).contains("не удалось создать изображение"), "{}", stderr(&output));
    assert!(!png.exists());
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
fn reverse_mode_prints_dependants_by_levels() {
    let output = run_app(&["tests/fixtures/reverse_demo_g.yaml"]);
    assert!(output.status.success());
    let text = stdout(&output);
    assert!(text.starts_with("Обратные зависимости G в графе A: 5\n"), "{text}");
    assert!(text.contains("уровень 1 (напрямую): D, E"), "{text}");
}

#[test]
fn reverse_mode_handles_cycles() {
    let output = run_app(&["tests/fixtures/reverse_cycle_a.yaml"]);
    assert!(output.status.success());
    assert!(stdout(&output).contains("уровень 2 (через другие пакеты): B"));
}

#[test]
fn reverse_mode_reports_package_outside_the_graph() {
    let output = run_app(&["tests/fixtures/reverse_not_in_graph.yaml"]);
    assert!(!output.status.success());
    assert!(stderr(&output).contains("'Z'"), "{}", stderr(&output));
}

#[test]
fn reverse_mode_validates_the_package_name() {
    let output = run_app(&["tests/fixtures/reverse_bad_name.yaml"]);
    assert!(!output.status.success());
    assert!(stderr(&output).contains("reverse_for"), "{}", stderr(&output));
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
