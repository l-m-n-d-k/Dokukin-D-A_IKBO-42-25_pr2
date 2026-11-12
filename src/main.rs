//! Точка входа CLI-приложения `depgraph`.

use std::process::ExitCode;

use depgraph::cli::{self, Command};
use depgraph::config;
use depgraph::error::AppError;
use depgraph::render::{DEFAULT_RENDER_URL, MermaidInk, RENDER_URL_ENV};
use depgraph::source::report_for;

/// Запускает приложение и превращает ошибку в сообщение и код возврата 1.
fn main() -> ExitCode {
    match run(std::env::args().skip(1)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Ошибка: {error}");
            ExitCode::FAILURE
        }
    }
}

/// Разбирает аргументы, загружает конфигурацию, строит граф зависимостей
/// заданного пакета и выводит его в виде Mermaid-текста с сохранением PNG
/// или (если задан `reverse_for`) выводит обратные зависимости.
fn run(args: impl Iterator<Item = String>) -> Result<(), AppError> {
    match cli::parse_args(args)? {
        Command::Help => println!("{}", cli::USAGE),
        Command::Run(path) => {
            let config = config::load_config(&path)?;
            let url = std::env::var(RENDER_URL_ENV).unwrap_or(DEFAULT_RENDER_URL.to_string());
            report_for(&config, &MermaidInk::new(&url), &mut std::io::stdout())?;
        }
    }
    Ok(())
}
