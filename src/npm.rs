//! Получение сведений о пакетах из репозитория npm.
//!
//! Репозиторий npm — это обычный HTTP-сервер. По адресу `<репозиторий>/<имя>`
//! он отдаёт JSON-документ со всеми версиями пакета: у каждой версии есть
//! поле `dependencies` — прямые зависимости в виде «имя пакета — допустимый
//! диапазон версий». Кроме того, в документе есть метки версий (`dist-tags`),
//! например `latest`.
//!
//! Менеджеры пакетов не используются: документ скачивается по HTTP,
//! а версия и зависимости выбираются кодом проекта.

use std::collections::BTreeMap;
use std::time::Duration;

use serde_json::{Map, Value};

use crate::graph::{DependencySource, Package, Requirement};
use crate::semver::resolve_version;

/// Прямые зависимости: имя пакета → диапазон версий.
/// `BTreeMap` хранит ключи отсортированными, поэтому вывод стабилен.
pub type Dependencies = BTreeMap<String, String>;

/// Имя поля с прямыми зависимостями.
const DEPENDENCIES_FIELD: &str = "dependencies";

/// Имя поля с версиями пакета в документе репозитория.
const VERSIONS_FIELD: &str = "versions";

/// Имя поля с метками версий в документе репозитория.
const TAGS_FIELD: &str = "dist-tags";

/// Сокращённый формат ответа репозитория: только то, что нужно для установки
/// (версии и зависимости), поэтому документ в разы меньше полного.
const ACCEPT_HEADER: &str = "application/vnd.npm.install-v1+json";

/// Таймаут всего HTTP-запроса.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(15);

/// HTTP-статус «не найдено».
const HTTP_NOT_FOUND: u16 = 404;

/// Сведения обо всех версиях одного пакета.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Packument {
    /// Метки версий: `latest` → `1.3.8` и т. п.
    pub dist_tags: BTreeMap<String, String>,
    /// Все версии пакета и их прямые зависимости.
    pub versions: BTreeMap<String, Dependencies>,
}

/// Строит адрес документа пакета: `<репозиторий>/<имя>`.
///
/// Лишний `/` в конце адреса репозитория отбрасывается. Имя с областью
/// (`@scope/name`) вставляется как есть: репозиторий npm понимает такой путь.
pub fn packument_url(repository: &str, name: &str) -> String {
    format!("{}/{name}", repository.trim_end_matches('/'))
}

/// Достаёт прямые зависимости из JSON-объекта с полем `dependencies`.
/// Если поля нет, у пакета нет прямых зависимостей.
fn extract_dependencies(object: &Map<String, Value>) -> Result<Dependencies, String> {
    let Some(field) = object.get(DEPENDENCIES_FIELD) else {
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

/// Достаёт прямые зависимости из текста манифеста `package.json` одной версии.
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
    extract_dependencies(root)
}

/// Разбирает метки версий: объект «метка → версия». Поле необязательно.
fn parse_tags(root: &Map<String, Value>) -> Result<BTreeMap<String, String>, String> {
    let Some(field) = root.get(TAGS_FIELD) else {
        return Ok(BTreeMap::new());
    };
    let Some(map) = field.as_object() else {
        return Err(format!("поле '{TAGS_FIELD}' должно быть объектом"));
    };
    map.iter()
        .map(|(tag, version)| {
            let version = version.as_str().ok_or(format!("метка '{tag}' должна быть строкой"))?;
            Ok((tag.clone(), version.to_string()))
        })
        .collect()
}

/// Разбирает документ пакета: все версии с зависимостями и метки версий.
///
/// # Ошибки
///
/// Возвращает текст ошибки, если документ не JSON, в нём нет поля `versions`
/// или поля имеют неверный формат.
pub fn parse_packument(text: &str) -> Result<Packument, String> {
    let root: Value =
        serde_json::from_str(text).map_err(|e| format!("ответ не является JSON: {e}"))?;
    let Some(root) = root.as_object() else {
        return Err("документ пакета должен быть JSON-объектом".to_string());
    };
    let Some(versions) = root.get(VERSIONS_FIELD).and_then(Value::as_object) else {
        return Err(format!("в документе пакета нет поля '{VERSIONS_FIELD}'"));
    };

    let mut packument = Packument { dist_tags: parse_tags(root)?, ..Packument::default() };
    for (version, manifest) in versions {
        let Some(manifest) = manifest.as_object() else {
            return Err(format!("описание версии '{version}' должно быть объектом"));
        };
        let dependencies = extract_dependencies(manifest)?;
        packument.versions.insert(version.clone(), dependencies);
    }
    Ok(packument)
}

/// Создаёт HTTP-клиент с таймаутом. Один клиент можно использовать для многих
/// запросов: соединения с сервером переиспользуются.
fn new_agent() -> ureq::Agent {
    ureq::Agent::config_builder().timeout_global(Some(REQUEST_TIMEOUT)).build().into()
}

/// Скачивает документ по адресу `url` и возвращает его текст.
///
/// # Ошибки
///
/// Возвращает понятное пользователю описание: пакет не найден,
/// сервер вернул ошибку или недоступен.
pub fn fetch_manifest(url: &str) -> Result<String, String> {
    fetch_with(&new_agent(), url)
}

/// Скачивает документ по адресу `url` с помощью готового клиента.
fn fetch_with(agent: &ureq::Agent, url: &str) -> Result<String, String> {
    let request = agent.get(url).header("Accept", ACCEPT_HEADER);
    let mut response = request.call().map_err(|e| describe_http_error(&e, url))?;
    response
        .body_mut()
        .read_to_string()
        .map_err(|e| format!("не удалось прочитать ответ сервера {url}: {e}"))
}

/// Превращает ошибку HTTP-клиента в сообщение для пользователя.
fn describe_http_error(error: &ureq::Error, url: &str) -> String {
    match error {
        ureq::Error::StatusCode(HTTP_NOT_FOUND) => {
            format!("пакет не найден в репозитории ({url})")
        }
        ureq::Error::StatusCode(code) => format!("сервер вернул статус {code} ({url})"),
        other => format!("не удалось подключиться к {url}: {other}"),
    }
}

/// Реальный репозиторий npm, доступный по HTTP.
#[derive(Debug, Clone)]
pub struct NpmRepository {
    /// Адрес репозитория, например `https://registry.npmjs.org`.
    repository: String,
    /// HTTP-клиент, общий для всех запросов этого репозитория.
    agent: ureq::Agent,
}

impl NpmRepository {
    /// Создаёт источник данных для репозитория по адресу `repository`.
    pub fn new(repository: &str) -> Self {
        Self { repository: repository.to_string(), agent: new_agent() }
    }
}

impl DependencySource for NpmRepository {
    /// Скачивает документ пакета, выбирает версию по `spec` и возвращает
    /// её прямые зависимости.
    fn package(&mut self, name: &str, spec: &str) -> Result<Package, String> {
        let text = fetch_with(&self.agent, &packument_url(&self.repository, name))?;
        let packument = parse_packument(&text)?;

        let available: Vec<String> = packument.versions.keys().cloned().collect();
        let version = resolve_version(spec, &available, &packument.dist_tags)?;
        let Some(dependencies) = packument.versions.get(&version) else {
            return Err(format!("версия '{version}' не найдена в репозитории"));
        };

        let dependencies: Vec<Requirement> =
            dependencies.iter().map(|(n, r)| (n.clone(), r.clone())).collect();
        Ok(Package { version, dependencies })
    }
}
