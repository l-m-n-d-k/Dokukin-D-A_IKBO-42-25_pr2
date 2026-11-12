//! Выбор версии пакета по диапазону в стиле npm.
//!
//! Зависимости в `package.json` записаны не точными версиями, а диапазонами:
//! `^1.2.3`, `~1.3.8`, `>=2.0.0 <3.0.0`, `1.x`, `*`, `1.0.0 - 2.0.0`,
//! `^1.0.0 || ^2.0.0`. Чтобы получить зависимости следующего уровня, нужно
//! выбрать из доступных версий самую новую, подходящую под диапазон.
//!
//! Поддерживается основная часть версии `MAJOR.MINOR.PATCH`. Предварительные
//! версии (`1.0.0-beta.1`) в диапазонах не участвуют, их можно выбрать только
//! точным указанием. Диапазоны, которые не относятся к версиям
//! (адреса git, пути к файлам, `npm:` и т. п.), не поддерживаются.

use std::collections::BTreeMap;

/// Основная часть версии: `(MAJOR, MINOR, PATCH)`.
pub type Triple = (u64, u64, u64);

/// Нулевое значение номера версии.
const ZERO: u64 = 0;

/// Метка версии, которую npm предпочитает при выборе подходящей версии.
const LATEST_TAG: &str = "latest";

/// Операторы сравнения, которые могут стоять отдельно от номера версии.
const OPERATORS: [&str; 8] = [">=", "<=", ">", "<", "=", "^", "~", "~>"];

/// Условие на версию. Диапазон из нескольких условий означает «все сразу».
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Cond {
    /// Версия не меньше указанной.
    Ge(Triple),
    /// Версия строго больше указанной.
    Gt(Triple),
    /// Версия строго меньше указанной.
    Lt(Triple),
}

impl Cond {
    /// Проверяет, удовлетворяет ли версия условию.
    fn holds(self, version: Triple) -> bool {
        match self {
            Self::Ge(bound) => version >= bound,
            Self::Gt(bound) => version > bound,
            Self::Lt(bound) => version < bound,
        }
    }
}

/// Версия, в которой часть номеров может быть не указана (`1`, `1.2`, `1.x`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Partial {
    major: Option<u64>,
    minor: Option<u64>,
    patch: Option<u64>,
}

impl Partial {
    /// Нижняя граница: неуказанные номера считаются нулями.
    fn lower(self) -> Triple {
        (self.major.unwrap_or(ZERO), self.minor.unwrap_or(ZERO), self.patch.unwrap_or(ZERO))
    }

    /// Первая версия, которая уже не подходит под частичную версию:
    /// у `1.2` это `1.3.0`, у `1` — `2.0.0`, у `1.2.3` — `1.2.4`.
    /// Для `*` такой версии нет.
    fn next(self) -> Option<Triple> {
        let (major, minor, patch) = self.lower();
        match (self.major, self.minor, self.patch) {
            (None, _, _) => None,
            (Some(_), None, _) => Some((major + 1, ZERO, ZERO)),
            (Some(_), Some(_), None) => Some((major, minor + 1, ZERO)),
            (Some(_), Some(_), Some(_)) => Some((major, minor, patch + 1)),
        }
    }

    /// Условия для «x-диапазона»: `1.2` = `>=1.2.0 <1.3.0`, `*` = любая версия.
    fn exact_conds(self) -> Vec<Cond> {
        let mut conds = Vec::new();
        if self.major.is_some() {
            conds.push(Cond::Ge(self.lower()));
        }
        conds.extend(self.next().map(Cond::Lt));
        conds
    }

    /// Условия для `^`: изменять нельзя самый левый ненулевой номер.
    fn caret_conds(self) -> Vec<Cond> {
        let (major, minor, patch) = self.lower();
        let upper = match (self.major, self.minor, self.patch) {
            (None, _, _) => return Vec::new(),
            (Some(ZERO), None, _) => (1, ZERO, ZERO),
            (Some(ZERO), Some(ZERO), None) => (ZERO, 1, ZERO),
            (Some(ZERO), Some(ZERO), Some(_)) => (ZERO, ZERO, patch + 1),
            (Some(ZERO), Some(_), _) => (ZERO, minor + 1, ZERO),
            (Some(_), _, _) => (major + 1, ZERO, ZERO),
        };
        vec![Cond::Ge(self.lower()), Cond::Lt(upper)]
    }

    /// Условия для `~`: можно менять патч (или минорную часть, если она не указана).
    fn tilde_conds(self) -> Vec<Cond> {
        let (major, minor, _) = self.lower();
        let upper = match (self.major, self.minor) {
            (None, _) => return Vec::new(),
            (Some(_), None) => (major + 1, ZERO, ZERO),
            (Some(_), Some(_)) => (major, minor + 1, ZERO),
        };
        vec![Cond::Ge(self.lower()), Cond::Lt(upper)]
    }
}

/// Разбирает номер версии. `x`, `X` и `*` означают «любой».
fn parse_number(text: &str) -> Result<Option<u64>, String> {
    if matches!(text, "x" | "X" | "*") {
        return Ok(None);
    }
    text.parse::<u64>().map(Some).map_err(|_| format!("'{text}' не является номером версии"))
}

/// Разбирает частичную версию вида `1`, `1.2`, `1.2.3`, `1.x`.
/// Предварительная часть (`-beta`) и сборка (`+build`) отбрасываются.
fn parse_partial(text: &str) -> Result<Partial, String> {
    let core = text.trim_start_matches(['v', '=']);
    let core = core.split(['-', '+']).next().unwrap_or_default();
    let mut parts = core.split('.');
    let mut next_number = || parts.next().filter(|p| !p.is_empty()).map(parse_number);
    let major = next_number().transpose()?.flatten();
    let minor = next_number().transpose()?.flatten();
    let patch = next_number().transpose()?.flatten();
    if parts.next().is_some() {
        return Err(format!("слишком много частей в версии '{text}'"));
    }
    Ok(Partial { major, minor, patch })
}

/// Условия для `>`: `>1.2` означает `>=1.3.0`, `>1.2.3` — строго больше.
fn greater_conds(partial: Partial) -> Vec<Cond> {
    match (partial.major, partial.minor, partial.patch) {
        (Some(_), Some(_), Some(_)) => vec![Cond::Gt(partial.lower())],
        _ => partial.next().map(Cond::Ge).into_iter().collect(),
    }
}

/// Условия для `<=`: `<=1.2` означает `<1.3.0`.
fn less_equal_conds(partial: Partial) -> Vec<Cond> {
    match partial.next() {
        Some(next) => vec![Cond::Lt(next)],
        None => Vec::new(),
    }
}

/// Разбирает один компаратор (`^1.2.3`, `>=1.0`, `1.x`) в список условий.
fn parse_comparator(text: &str) -> Result<Vec<Cond>, String> {
    let conds = if let Some(rest) = text.strip_prefix(">=") {
        vec![Cond::Ge(parse_partial(rest)?.lower())]
    } else if let Some(rest) = text.strip_prefix("<=") {
        less_equal_conds(parse_partial(rest)?)
    } else if let Some(rest) = text.strip_prefix('>') {
        greater_conds(parse_partial(rest)?)
    } else if let Some(rest) = text.strip_prefix('<') {
        vec![Cond::Lt(parse_partial(rest)?.lower())]
    } else if let Some(rest) = text.strip_prefix('^') {
        parse_partial(rest)?.caret_conds()
    } else if let Some(rest) = text.strip_prefix('~') {
        parse_partial(rest.trim_start_matches('>'))?.tilde_conds()
    } else {
        parse_partial(text)?.exact_conds()
    };
    Ok(conds)
}

/// Разбирает дефисный диапазон `A - B`: `>=A` и `<=B` (для частичной `B` — `<next`).
fn parse_hyphen(from: &str, to: &str) -> Result<Vec<Cond>, String> {
    let mut conds = vec![Cond::Ge(parse_partial(from)?.lower())];
    conds.extend(less_equal_conds(parse_partial(to)?));
    Ok(conds)
}

/// Разбирает набор условий, записанных через пробел (без `||`).
fn parse_set(text: &str) -> Result<Vec<Cond>, String> {
    let words: Vec<&str> = text.split_whitespace().collect();
    if let [from, "-", to] = words.as_slice() {
        return parse_hyphen(from, to);
    }
    let mut conds = Vec::new();
    for word in join_operators(&words) {
        conds.extend(parse_comparator(&word)?);
    }
    Ok(conds)
}

/// Склеивает оператор с числом, если между ними пробел: `>= 1.2.3` → `>=1.2.3`.
fn join_operators(words: &[&str]) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();
    let mut pending = String::new();
    for word in words {
        if OPERATORS.contains(word) {
            pending.push_str(word);
        } else {
            result.push(format!("{pending}{word}"));
            pending.clear();
        }
    }
    if !pending.is_empty() {
        result.push(pending);
    }
    result
}

/// Разбирает диапазон целиком: наборы условий, разделённые `||`.
/// Пустой диапазон означает «любая версия».
fn parse_range(text: &str) -> Result<Vec<Vec<Cond>>, String> {
    text.split("||")
        .map(|set| parse_set(set).map_err(|e| format!("неподдерживаемый диапазон '{text}': {e}")))
        .collect()
}

/// Разбирает номер версии `MAJOR.MINOR.PATCH` без предварительной части.
/// Для предварительных версий (`1.0.0-beta.1`) возвращает `None`.
fn parse_stable(text: &str) -> Option<Triple> {
    let partial = parse_partial(text).ok()?;
    let is_stable = !text.contains(['-', '+']);
    match (partial.major, partial.minor, partial.patch) {
        (Some(major), Some(minor), Some(patch)) if is_stable => Some((major, minor, patch)),
        _ => None,
    }
}

/// Выбирает версию по диапазону (или метке вроде `latest`).
///
/// Порядок выбора:
/// 1. метка из `dist_tags` (`latest`, `next`);
/// 2. точное совпадение с одной из доступных версий (в том числе предварительной);
/// 3. версия с меткой `latest`, если она подходит под диапазон (так выбирает
///    и npm: метка важнее более новой, но не отмеченной версии);
/// 4. самая новая стабильная версия, подходящая под диапазон.
///
/// # Ошибки
///
/// Возвращает описание проблемы: диапазон не поддерживается или ни одна из
/// доступных версий ему не подходит.
pub fn resolve_version(
    spec: &str,
    available: &[String],
    dist_tags: &BTreeMap<String, String>,
) -> Result<String, String> {
    let spec = spec.trim();
    if let Some(tagged) = dist_tags.get(spec) {
        return Ok(tagged.clone());
    }
    if available.iter().any(|v| v == spec) {
        return Ok(spec.to_string());
    }
    let sets = parse_range(spec)?;
    let matches = |triple: &Triple| sets.iter().any(|set| set.iter().all(|c| c.holds(*triple)));

    // Как и npm, предпочитаем версию с меткой `latest`, если она подходит.
    let latest = dist_tags.get(LATEST_TAG).filter(|v| parse_stable(v).is_some_and(|t| matches(&t)));
    if let Some(latest) = latest {
        return Ok(latest.clone());
    }
    available
        .iter()
        .filter_map(|v| parse_stable(v).map(|triple| (triple, v)))
        .filter(|(triple, _)| matches(triple))
        .max_by_key(|(triple, _)| *triple)
        .map(|(_, version)| version.clone())
        .ok_or_else(|| format!("нет версии, подходящей под '{spec}'"))
}
