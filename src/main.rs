//! Точка входа CLI-приложения `depgraph`.

use std::process::ExitCode;

use depgraph::cli::{self, Command};
use depgraph::config;
use depgraph::error::AppError;
use depgraph::npm;

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

/// Разбирает аргументы, загружает конфигурацию, получает прямые зависимости
/// заданного пакета и выводит их на экран.
fn run(args: impl Iterator<Item = String>) -> Result<(), AppError> {
    match cli::parse_args(args)? {
        Command::Help => println!("{}", cli::USAGE),
        Command::Run(path) => {
            let config = config::load_config(&path)?;
            let deps = npm::fetch_dependencies(&config)?;
            print!("{}", npm::format_dependencies(&config.package_name, &config.version, &deps));
        }
    }
    Ok(())
}
