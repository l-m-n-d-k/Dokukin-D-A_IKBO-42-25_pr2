//! `depgraph` — инструмент визуализации графа зависимостей npm-пакетов.
//!
//! Этап 1: минимальный CLI-прототип, настраиваемый через YAML-файл.
//! Этап 2: сбор данных — прямые зависимости пакета из репозитория npm.
//! Этап 3: транзитивный граф зависимостей (BFS), тестовый репозиторий.

pub mod cli;
pub mod config;
pub mod error;
pub mod graph;
pub mod npm;
pub mod semver;
pub mod source;
pub mod test_repo;
pub mod validate;
