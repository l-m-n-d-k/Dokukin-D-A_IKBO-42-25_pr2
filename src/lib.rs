//! `depgraph` — инструмент визуализации графа зависимостей npm-пакетов.
//!
//! Этап 1: минимальный CLI-прототип, настраиваемый через YAML-файл.
//! Этап 2: сбор данных — прямые зависимости пакета из репозитория npm.

pub mod cli;
pub mod config;
pub mod error;
pub mod npm;
pub mod validate;
