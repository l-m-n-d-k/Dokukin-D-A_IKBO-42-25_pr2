//! Проверка значений параметров конфигурации.
//!
//! Каждый параметр проверяется отдельной функцией, которая возвращает
//! `Ok(())` или текст ошибки. [`validate_config`] запускает все проверки
//! и собирает все найденные ошибки, чтобы пользователь увидел их сразу.

use std::path::Path;

use crate::config::Config;
use crate::error::FieldError;

/// Максимальная длина имени npm-пакета (ограничение реестра npm).
pub const MAX_NPM_NAME_LEN: usize = 214;

/// Количество чисел в основной части версии: `MAJOR.MINOR.PATCH`.
const VERSION_CORE_PARTS: usize = 3;

/// Требуемое расширение файла с изображением графа.
const OUTPUT_EXTENSION: &str = "png";

/// Проверяет все параметры конфигурации.
///
/// # Ошибки
///
/// Возвращает список ошибок по всем параметрам с недопустимыми значениями.
pub fn validate_config(config: &Config) -> Result<(), Vec<FieldError>> {
    let checks = [
        ("package_name", check_package_name(&config.package_name, config.test_mode)),
        ("repository", check_repository(&config.repository, config.test_mode)),
        ("version", check_version(&config.version)),
        ("output_file", check_output_file(&config.output_file)),
        ("reverse_for", check_reverse_for(config.reverse_for.as_deref(), config.test_mode)),
    ];

    let errors: Vec<FieldError> = checks
        .into_iter()
        .filter_map(|(field, result)| result.err().map(|msg| FieldError::new(field, msg)))
        .collect();

    if errors.is_empty() { Ok(()) } else { Err(errors) }
}

/// Проверяет необязательный параметр `reverse_for`: если он задан, это должно
/// быть корректное имя пакета (по тем же правилам, что и `package_name`).
pub fn check_reverse_for(target: Option<&str>, test_mode: bool) -> Result<(), String> {
    target.map_or(Ok(()), |name| check_package_name(name, test_mode))
}

/// Проверяет имя пакета.
///
/// В обычном режиме имя должно соответствовать правилам npm,
/// в тестовом режиме — состоять из больших латинских букв.
pub fn check_package_name(name: &str, test_mode: bool) -> Result<(), String> {
    if name.is_empty() {
        return Err("имя пакета не может быть пустым".to_string());
    }
    if test_mode {
        check_test_package_name(name)
    } else {
        check_npm_package_name(name)
    }
}

/// Проверяет имя пакета тестового репозитория: только `A`–`Z`.
pub fn check_test_package_name(name: &str) -> Result<(), String> {
    if name.chars().all(|c| c.is_ascii_uppercase()) {
        Ok(())
    } else {
        Err(format!(
            "в тестовом режиме имя пакета должно состоять из больших латинских букв \
             (например, A или ABC), получено '{name}'"
        ))
    }
}

/// Проверяет имя пакета по правилам npm: длина, нижний регистр,
/// допустимые символы и формат `@scope/name` для пакетов с областью.
pub fn check_npm_package_name(name: &str) -> Result<(), String> {
    if name.len() > MAX_NPM_NAME_LEN {
        return Err(format!("имя пакета длиннее {MAX_NPM_NAME_LEN} символов"));
    }
    if name.chars().any(|c| c.is_uppercase()) {
        return Err(format!("имя npm-пакета должно быть в нижнем регистре: '{name}'"));
    }

    match name.strip_prefix('@') {
        Some(scoped) => check_scoped_name(scoped),
        None => check_name_part(name),
    }
}

/// Проверяет имя вида `scope/name` (символ `@` уже отброшен).
fn check_scoped_name(scoped: &str) -> Result<(), String> {
    let Some((scope, package)) = scoped.split_once('/') else {
        return Err("имя с областью должно иметь вид @scope/name".to_string());
    };
    check_name_part(scope)?;
    check_name_part(package)
}

/// Проверяет одну часть имени npm-пакета (область или собственно имя).
fn check_name_part(part: &str) -> Result<(), String> {
    if part.is_empty() {
        return Err("часть имени пакета не может быть пустой".to_string());
    }
    if part.starts_with('.') || part.starts_with('_') {
        return Err(format!("имя '{part}' не может начинаться с '.' или '_'"));
    }
    match part.chars().find(|&c| !is_npm_name_char(c)) {
        Some(bad) => Err(format!("недопустимый символ '{bad}' в имени '{part}'")),
        None => Ok(()),
    }
}

/// Символы, разрешённые в имени npm-пакета: `a-z`, `0-9`, `-`, `.`, `_`, `~`.
fn is_npm_name_char(c: char) -> bool {
    c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '-' | '.' | '_' | '~')
}

/// Проверяет источник данных.
///
/// В обычном режиме это должен быть URL, в тестовом — путь к
/// существующему файлу с описанием тестового репозитория.
pub fn check_repository(repository: &str, test_mode: bool) -> Result<(), String> {
    if repository.is_empty() {
        return Err("адрес репозитория не может быть пустым".to_string());
    }
    if test_mode {
        check_test_repository_file(repository)
    } else {
        check_url(repository)
    }
}

/// Проверяет, что строка — URL со схемой `http`/`https` и непустым хостом.
pub fn check_url(url: &str) -> Result<(), String> {
    let Some(rest) = url.strip_prefix("https://").or_else(|| url.strip_prefix("http://")) else {
        return Err(format!(
            "URL должен начинаться с http:// или https://, получено '{url}' \
             (для файла тестового репозитория укажите test_mode: true)"
        ));
    };
    if url.chars().any(char::is_whitespace) {
        return Err(format!("URL не должен содержать пробелов: '{url}'"));
    }
    let host = rest.split('/').next().unwrap_or_default();
    if host.is_empty() {
        return Err(format!("в URL не указан хост: '{url}'"));
    }
    Ok(())
}

/// Проверяет, что путь указывает на существующий файл.
pub fn check_test_repository_file(path: &str) -> Result<(), String> {
    let path_ref = Path::new(path);
    if !path_ref.exists() {
        return Err(format!("файл тестового репозитория не найден: '{path}'"));
    }
    if !path_ref.is_file() {
        return Err(format!("путь к тестовому репозиторию не является файлом: '{path}'"));
    }
    Ok(())
}

/// Проверяет версию по правилам Semantic Versioning:
/// `MAJOR.MINOR.PATCH[-pre-release][+build]`, например `4.18.2` или `1.0.0-beta.1`.
pub fn check_version(version: &str) -> Result<(), String> {
    if version.is_empty() {
        return Err("версия не может быть пустой".to_string());
    }
    let (without_build, build) = split_optional(version, '+');
    let (core, pre_release) = split_optional(without_build, '-');

    check_version_core(core).map_err(|msg| format!("{msg} (версия '{version}')"))?;
    if let Some(pre) = pre_release {
        check_identifiers(pre, "pre-release")?;
    }
    if let Some(build) = build {
        check_identifiers(build, "build")?;
    }
    Ok(())
}

/// Делит строку по первому вхождению `separator`.
/// Если разделителя нет, вторая часть — `None`.
fn split_optional(text: &str, separator: char) -> (&str, Option<&str>) {
    match text.split_once(separator) {
        Some((head, tail)) => (head, Some(tail)),
        None => (text, None),
    }
}

/// Проверяет основную часть версии: три неотрицательных числа через точку
/// без ведущих нулей.
fn check_version_core(core: &str) -> Result<(), String> {
    let parts: Vec<&str> = core.split('.').collect();
    if parts.len() != VERSION_CORE_PARTS {
        return Err("версия должна иметь вид MAJOR.MINOR.PATCH".to_string());
    }
    for part in parts {
        if part.is_empty() || !part.chars().all(|c| c.is_ascii_digit()) {
            return Err(format!("'{part}' не является числом"));
        }
        if part.starts_with('0') && part != "0" {
            return Err(format!("число '{part}' не должно начинаться с нуля"));
        }
    }
    Ok(())
}

/// Проверяет идентификаторы pre-release/build: непустые части через точку,
/// состоящие из латинских букв, цифр и дефиса.
fn check_identifiers(text: &str, kind: &str) -> Result<(), String> {
    let is_valid = |id: &str| {
        !id.is_empty() && id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
    };
    if text.split('.').all(is_valid) {
        Ok(())
    } else {
        Err(format!("некорректная {kind}-часть версии: '{text}'"))
    }
}

/// Проверяет имя выходного файла: расширение `.png`, путь не является
/// каталогом, родительский каталог существует.
pub fn check_output_file(output: &str) -> Result<(), String> {
    if output.is_empty() {
        return Err("имя выходного файла не может быть пустым".to_string());
    }
    let path = Path::new(output);
    let has_png_ext = path
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case(OUTPUT_EXTENSION));
    if !has_png_ext {
        return Err(format!("файл должен иметь расширение .{OUTPUT_EXTENSION}: '{output}'"));
    }
    if path.is_dir() {
        return Err(format!("'{output}' является каталогом, а не файлом"));
    }
    match path.parent() {
        Some(dir) if !dir.as_os_str().is_empty() && !dir.is_dir() => {
            Err(format!("каталог для выходного файла не существует: '{}'", dir.display()))
        }
        _ => Ok(()),
    }
}
