//! Выбор источника данных по конфигурации и построение графа.

use std::fs;
use std::io::Write;
use std::path::Path;

use crate::config::Config;
use crate::error::AppError;
use crate::graph::{DependencySource, Graph, build_graph};
use crate::mermaid::to_mermaid;
use crate::npm::NpmRepository;
use crate::render::Renderer;
use crate::reverse::{format_reverse, reverse_dependencies};
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

/// Выполняет работу приложения по конфигурации и пишет результат в `out`.
///
/// Строится граф зависимостей пакета `package_name`. Дальше есть два режима:
/// - задан `reverse_for` — выводятся обратные зависимости этого пакета;
/// - иначе — выводится Mermaid-текст графа, а изображение графа сохраняется
///   в PNG-файл `output_file` (рисует `renderer`).
///
/// Mermaid-текст выводится до отрисовки, поэтому при сбое сервиса отрисовки
/// схему всё равно можно взять из вывода.
///
/// # Ошибки
///
/// - [`AppError::Npm`] — не удалось получить или разобрать данные;
/// - [`AppError::Reverse`] — пакета `reverse_for` нет в графе;
/// - [`AppError::Render`] — не удалось нарисовать или сохранить изображение.
pub fn report_for(
    config: &Config,
    renderer: &dyn Renderer,
    out: &mut dyn Write,
) -> Result<(), AppError> {
    let graph = build_graph_for(config)?;
    if let Some(target) = &config.reverse_for {
        let levels = reverse_dependencies(&graph, target).map_err(AppError::Reverse)?;
        return write_text(out, &format_reverse(&graph, target, &levels));
    }

    let mermaid = to_mermaid(&graph);
    write_text(out, &mermaid)?;
    let png = renderer.render(&mermaid).map_err(AppError::Render)?;
    let path = Path::new(&config.output_file);
    fs::write(path, png)
        .map_err(|e| AppError::Render(format!("не удалось записать '{}': {e}", path.display())))?;
    write_text(out, &format!("Изображение графа сохранено в файле {}\n", config.output_file))
}

/// Пишет текст в `out`, превращая ошибку записи в [`AppError::Render`].
fn write_text(out: &mut dyn Write, text: &str) -> Result<(), AppError> {
    out.write_all(text.as_bytes()).map_err(|e| AppError::Render(format!("ошибка вывода: {e}")))
}
