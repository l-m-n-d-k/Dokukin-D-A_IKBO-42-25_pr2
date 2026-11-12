//! Выбор источника данных по конфигурации и построение графа.

use std::path::Path;

use crate::config::Config;
use crate::error::AppError;
use crate::graph::{DependencySource, Graph, build_graph};
use crate::npm::NpmRepository;
use crate::test_repo::load_test_repository;

/// Открывает источник данных: тестовый файл (`test_mode: true`)
/// или репозиторий npm по URL.
///
/// # Ошибки
///
/// Возвращает [`AppError::Npm`], если тестовый файл не удалось прочитать
/// или он некорректен.
pub fn open_source(config: &Config) -> Result<Box<dyn DependencySource>, AppError> {
    if config.test_mode {
        let repository = load_test_repository(Path::new(&config.repository))
            .map_err(AppError::Npm)?;
        Ok(Box::new(repository))
    } else {
        Ok(Box::new(NpmRepository::new(&config.repository)))
    }
}

/// Строит граф зависимостей пакета из конфигурации.
///
/// # Ошибки
///
/// Возвращает [`AppError::Npm`], если данные не удалось получить или разобрать.
pub fn build_graph_for(config: &Config) -> Result<Graph, AppError> {
    let mut source = open_source(config)?;
    build_graph(source.as_mut(), &config.package_name, &config.version).map_err(AppError::Npm)
}
