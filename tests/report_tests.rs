//! Тесты итоговой работы приложения: вывод текста и сохранение PNG.

mod common;

use std::fs;

use common::{fake_png, temp_path};
use depgraph::config::Config;
use depgraph::error::AppError;
use depgraph::render::Renderer;
use depgraph::source::report_for;

/// Рендерер-заглушка: возвращает заранее заданный результат.
struct FakeRenderer(Result<Vec<u8>, String>);

impl Renderer for FakeRenderer {
    fn render(&self, _mermaid: &str) -> Result<Vec<u8>, String> {
        self.0.clone()
    }
}

/// Конфигурация тестового режима для файла из `tests/fixtures/repos`.
fn config_for(repo: &str, output: &str, reverse_for: Option<&str>) -> Config {
    Config {
        package_name: "A".to_string(),
        repository: format!("tests/fixtures/repos/{repo}"),
        test_mode: true,
        version: "1.0.0".to_string(),
        output_file: output.to_string(),
        reverse_for: reverse_for.map(str::to_string),
    }
}

/// Запускает работу приложения и возвращает вывод и результат.
fn run(config: &Config, renderer: &FakeRenderer) -> (String, Result<(), AppError>) {
    let mut out = Vec::new();
    let result = report_for(config, renderer, &mut out);
    (String::from_utf8(out).unwrap(), result)
}

#[test]
fn mermaid_text_is_printed_and_png_is_saved() {
    let output = temp_path("diamond.png");
    let config = config_for("diamond.txt", output.to_str().unwrap(), None);

    let (text, result) = run(&config, &FakeRenderer(Ok(fake_png())));

    result.unwrap();
    assert!(text.starts_with("graph TD\n"), "{text}");
    assert!(text.contains("n1 --> n3"), "{text}");
    assert!(text.contains("Изображение графа сохранено"), "{text}");
    assert_eq!(fs::read(&output).unwrap(), fake_png());
    fs::remove_file(&output).unwrap();
}

#[test]
fn mermaid_text_is_printed_even_if_rendering_fails() {
    let output = temp_path("failed.png");
    let config = config_for("diamond.txt", output.to_str().unwrap(), None);

    let (text, result) = run(&config, &FakeRenderer(Err("сервис занят".to_string())));

    assert!(text.starts_with("graph TD\n"), "{text}");
    assert!(matches!(&result, Err(AppError::Render(m)) if m.contains("занят")), "{result:?}");
    assert!(!output.exists());
}

#[test]
fn unwritable_output_file_is_reported() {
    let output = temp_path("no_such_dir").join("graph.png");
    let config = config_for("diamond.txt", output.to_str().unwrap(), None);

    let (_, result) = run(&config, &FakeRenderer(Ok(fake_png())));

    assert!(matches!(&result, Err(AppError::Render(m)) if m.contains("записать")), "{result:?}");
}

#[test]
fn reverse_mode_prints_dependants_and_draws_nothing() {
    let output = temp_path("reverse.png");
    let config = config_for("diamond.txt", output.to_str().unwrap(), Some("D"));

    let (text, result) = run(&config, &FakeRenderer(Err("не должен вызываться".to_string())));

    result.unwrap();
    assert!(text.starts_with("Обратные зависимости D в графе A: 3\n"), "{text}");
    assert!(!output.exists());
}

#[test]
fn unknown_reverse_package_is_reported() {
    let config = config_for("diamond.txt", "unused.png", Some("Z"));
    let (_, result) = run(&config, &FakeRenderer(Ok(fake_png())));
    assert!(matches!(result, Err(AppError::Reverse(_))));
}
