//! Тестовый репозиторий: граф зависимостей, описанный в текстовом файле.
//!
//! Пакеты в таком репозитории называются большими латинскими буквами
//! (`A`, `B`, `ABC`). Формат файла — по одной строке на пакет:
//!
//! ```text
//! # комментарий
//! A: B C
//! B: C
//! C:
//! ```
//!
//! Слева от двоеточия — пакет, справа — его прямые зависимости через пробел.
//! Пустые строки и строки-комментарии (`#`) пропускаются. Пакет, который
//! упомянут только как зависимость, считается пакетом без зависимостей.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use crate::graph::{DependencySource, Package};
use crate::validate::check_test_package_name;

/// Граф зависимостей тестового репозитория.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TestRepository {
    /// Пакет → его прямые зависимости (в порядке записи в файле).
    dependencies: BTreeMap<String, Vec<String>>,
    /// Все известные пакеты: описанные в файле и упомянутые как зависимость.
    known: BTreeSet<String>,
}

/// Читает тестовый репозиторий из файла.
///
/// # Ошибки
///
/// Возвращает описание проблемы: файл не прочитан или содержимое некорректно.
pub fn load_test_repository(path: &Path) -> Result<TestRepository, String> {
    let text = fs::read_to_string(path)
        .map_err(|e| format!("не удалось прочитать файл '{}': {e}", path.display()))?;
    parse_test_repository(&text).map_err(|e| format!("файл '{}': {e}", path.display()))
}

/// Разбирает текст тестового репозитория.
///
/// # Ошибки
///
/// Возвращает описание ошибки с номером строки: нет двоеточия, имя пакета
/// не из больших латинских букв или пакет описан дважды.
pub fn parse_test_repository(text: &str) -> Result<TestRepository, String> {
    let mut repository = TestRepository::default();
    for (index, line) in text.lines().enumerate() {
        let line = line.split('#').next().unwrap_or_default().trim();
        if line.is_empty() {
            continue;
        }
        repository.add_line(line).map_err(|e| format!("строка {}: {e}", index + 1))?;
    }
    Ok(repository)
}

impl TestRepository {
    /// Добавляет в репозиторий строку вида `A: B C`.
    fn add_line(&mut self, line: &str) -> Result<(), String> {
        let Some((name, rest)) = line.split_once(':') else {
            return Err(format!("ожидалась запись вида 'A: B C', получено '{line}'"));
        };
        let name = name.trim();
        check_test_package_name(name).and_then(|()| require_name(name))?;
        if self.dependencies.contains_key(name) {
            return Err(format!("пакет '{name}' описан повторно"));
        }

        let dependencies: Vec<String> = rest.split_whitespace().map(str::to_string).collect();
        for dependency in &dependencies {
            check_test_package_name(dependency)?;
        }
        self.known.insert(name.to_string());
        self.known.extend(dependencies.iter().cloned());
        self.dependencies.insert(name.to_string(), dependencies);
        Ok(())
    }
}

/// Проверяет, что имя пакета не пустое (пустое имя слева от `:` — ошибка).
fn require_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        Err("не указано имя пакета перед ':'".to_string())
    } else {
        Ok(())
    }
}

impl DependencySource for TestRepository {
    /// Возвращает зависимости пакета из файла. Версий в тестовом репозитории
    /// нет, поэтому `spec` игнорируется, а версия пустая.
    fn package(&mut self, name: &str, _spec: &str) -> Result<Package, String> {
        if !self.known.contains(name) {
            return Err("пакет не найден в тестовом репозитории".to_string());
        }
        let dependencies = self.dependencies.get(name).cloned().unwrap_or_default();
        Ok(Package {
            version: String::new(),
            dependencies: dependencies.into_iter().map(|d| (d, String::new())).collect(),
        })
    }
}
