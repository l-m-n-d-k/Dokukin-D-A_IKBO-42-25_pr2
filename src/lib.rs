//! `depgraph` — инструмент визуализации графа зависимостей npm-пакетов.
//!
//! Этап 1: минимальный CLI-прототип, настраиваемый через YAML-файл.

pub mod cli;
pub mod config;
pub mod error;
pub mod validate;
