//! Получение прямых зависимостей пакета npm.
//!
//! Репозиторий npm — это обычный HTTP-сервер. Метаданные одной версии пакета
//! лежат по адресу `<репозиторий>/<имя>/<версия>` в виде JSON-документа
//! (манифеста `package.json`). Поле `dependencies` манифеста содержит прямые
//! зависимости: пары «имя пакета — допустимый диапазон версий».
//!
//! Менеджеры пакетов не используются: манифест скачивается по HTTP,
//! а зависимости достаются из JSON кодом этого модуля.

use std::collections::BTreeMap;
use std::time::Duration;

use serde_json::Value;

use crate::config::Config;
use crate::error::AppError;

/// Прямые зависимости: имя пакета → диапазон версий.
/// `BTreeMap` хранит ключи отсортированными, поэтому вывод стабилен.
pub type Dependencies = BTreeMap<String, String>;

/// Имя поля манифеста с прямыми зависимостями.
const DEPENDENCIES_FIELD: &str = "dependencies";

/// Таймаут всего HTTP-запроса.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(15);

/// HTTP-статус «не найдено».
const HTTP_NOT_FOUND: u16 = 404;

/// Строит адрес манифеста: `<репозиторий>/<имя>/<версия>`.
///
/// Лишний `/` в конце адреса репозитория отбрасывается. Имя с областью
/// (`@scope/name`) вставляется как есть: реестр npm понимает такой путь.
pub fn manifest_url(repository: &str, name: &str, version: &str) -> String {
    format!("{}/{name}/{version}", repository.trim_end_matches('/'))
}

/// Достаёт прямые зависимости из текста манифеста `package.json`.
///
/// Если поля `dependencies` нет, у пакета нет прямых зависимостей,
/// и возвращается пустой набор.
///
/// # Ошибки
///
/// Возвращает текст ошибки, если текст не является JSON-объектом или если
/// `dependencies` не объект «строка → строка».
pub fn parse_dependencies(manifest: &str) -> Result<Dependencies, String> {
    let root: Value =
        serde_json::from_str(manifest).map_err(|e| format!("ответ не является JSON: {e}"))?;
    let Some(root) = root.as_object() else {
        return Err("манифест пакета должен быть JSON-объектом".to_string());
    };
    let Some(field) = root.get(DEPENDENCIES_FIELD) else {
        return Ok(Dependencies::new());
    };
    let Some(map) = field.as_object() else {
        return Err(format!("поле '{DEPENDENCIES_FIELD}' должно быть объектом"));
    };

    let mut result = Dependencies::new();
    for (name, range) in map {
        let Some(range) = range.as_str() else {
            return Err(format!("диапазон версий пакета '{name}' должен быть строкой"));
        };
        result.insert(name.clone(), range.to_string());
    }
    Ok(result)
}

/// Скачивает манифест по адресу `url` и возвращает его текст.
///
/// # Ошибки
///
/// Возвращает понятное пользователю описание: пакет или версия не найдены,
/// сервер вернул ошибку или недоступен.
pub fn fetch_manifest(url: &str) -> Result<String, String> {
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(REQUEST_TIMEOUT))
        .build()
        .into();

    let mut response = agent.get(url).call().map_err(|e| describe_http_error(&e, url))?;
    response
        .body_mut()
        .read_to_string()
        .map_err(|e| format!("не удалось прочитать ответ сервера {url}: {e}"))
}

/// Превращает ошибку HTTP-клиента в сообщение для пользователя.
fn describe_http_error(error: &ureq::Error, url: &str) -> String {
    match error {
        ureq::Error::StatusCode(HTTP_NOT_FOUND) => {
            format!("пакет или версия не найдены в репозитории ({url})")
        }
        ureq::Error::StatusCode(code) => format!("сервер вернул статус {code} ({url})"),
        other => format!("не удалось подключиться к {url}: {other}"),
    }
}

/// Получает прямые зависимости пакета из конфигурации.
///
/// # Ошибки
///
/// - [`AppError::Unsupported`] — включён тестовый режим (этап 3);
/// - [`AppError::Npm`] — не удалось скачать или разобрать манифест.
pub fn fetch_dependencies(config: &Config) -> Result<Dependencies, AppError> {
    if config.test_mode {
        return Err(AppError::Unsupported(
            "тестовый режим пока не реализован, он появится на этапе 3".to_string(),
        ));
    }
    let url = manifest_url(&config.repository, &config.package_name, &config.version);
    let manifest = fetch_manifest(&url).map_err(AppError::Npm)?;
    parse_dependencies(&manifest).map_err(AppError::Npm)
}

/// Форматирует зависимости для вывода на экран: заголовок и по строке
/// `имя: диапазон` на каждую зависимость.
pub fn format_dependencies(name: &str, version: &str, deps: &Dependencies) -> String {
    let mut text = format!("Прямые зависимости {name}@{version}: {}\n", deps.len());
    if deps.is_empty() {
        text.push_str("  (нет прямых зависимостей)\n");
    }
    for (dependency, range) in deps {
        text.push_str(&format!("  {dependency}: {range}\n"));
    }
    text
}
