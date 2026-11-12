//! Тесты выбора версии по диапазону.

use std::collections::BTreeMap;

use depgraph::semver::resolve_version;

/// Набор доступных версий для тестов.
fn versions() -> Vec<String> {
    [
        "0.0.3", "0.1.0", "0.1.5", "0.2.0", "1.0.0", "1.2.3", "1.2.9", "1.3.0", "1.10.0",
        "2.0.0", "2.1.2", "2.5.0", "3.0.0", "3.0.1-beta.1",
    ]
    .iter()
    .map(|v| v.to_string())
    .collect()
}

/// Выбирает версию из тестового набора без меток.
fn pick(spec: &str) -> Result<String, String> {
    resolve_version(spec, &versions(), &BTreeMap::new())
}

#[test]
fn exact_version() {
    assert_eq!(pick("1.2.3").unwrap(), "1.2.3");
    assert_eq!(pick("=1.2.3").unwrap(), "1.2.3");
    assert!(pick("1.2.4").is_err());
}

#[test]
fn caret_ranges() {
    assert_eq!(pick("^1.2.3").unwrap(), "1.10.0");
    assert_eq!(pick("^1.0.0").unwrap(), "1.10.0");
    assert_eq!(pick("^0.1.0").unwrap(), "0.1.5");
    assert_eq!(pick("^0.0.3").unwrap(), "0.0.3");
    assert_eq!(pick("^2").unwrap(), "2.5.0");
    assert_eq!(pick("^0").unwrap(), "0.2.0");
}

#[test]
fn tilde_ranges() {
    assert_eq!(pick("~1.2.3").unwrap(), "1.2.9");
    assert_eq!(pick("~1.2").unwrap(), "1.2.9");
    assert_eq!(pick("~1").unwrap(), "1.10.0");
    assert_eq!(pick("~2.1.0").unwrap(), "2.1.2");
}

#[test]
fn x_ranges_and_any() {
    assert_eq!(pick("1.x").unwrap(), "1.10.0");
    assert_eq!(pick("1.2.x").unwrap(), "1.2.9");
    assert_eq!(pick("1.2").unwrap(), "1.2.9");
    assert_eq!(pick("2").unwrap(), "2.5.0");
    assert_eq!(pick("*").unwrap(), "3.0.0");
    assert_eq!(pick("").unwrap(), "3.0.0");
    assert_eq!(pick("x").unwrap(), "3.0.0");
}

#[test]
fn comparators() {
    assert_eq!(pick(">=1.2.3 <1.3.0").unwrap(), "1.2.9");
    assert_eq!(pick(">=2.0.0").unwrap(), "3.0.0");
    assert_eq!(pick(">2.1.2 <3").unwrap(), "2.5.0");
    assert_eq!(pick("<1.0.0").unwrap(), "0.2.0");
    assert_eq!(pick("<=1.2.3").unwrap(), "1.2.3");
    assert_eq!(pick(">1").unwrap(), "3.0.0");
    assert_eq!(pick("<=1").unwrap(), "1.10.0");
}

#[test]
fn operators_may_be_separated_from_numbers() {
    assert_eq!(pick(">= 2.1.2 < 3").unwrap(), "2.5.0");
    assert_eq!(pick("^ 1.0.0").unwrap(), "1.10.0");
}

#[test]
fn hyphen_ranges() {
    assert_eq!(pick("1.0.0 - 1.2.9").unwrap(), "1.2.9");
    assert_eq!(pick("1.0.0 - 1.2").unwrap(), "1.2.9");
    assert_eq!(pick("1.2.3 - 2").unwrap(), "2.5.0");
}

#[test]
fn alternatives() {
    assert_eq!(pick("^0.1.0 || ^2.0.0").unwrap(), "2.5.0");
    assert_eq!(pick("^1.0.0 || ^0.2.0").unwrap(), "1.10.0");
    assert_eq!(pick("^9.0.0 || ^0.0.3").unwrap(), "0.0.3");
}

#[test]
fn newest_version_is_compared_as_number_not_text() {
    // "1.10.0" новее "1.3.0", хотя как строка меньше.
    assert_eq!(pick("^1.0.0").unwrap(), "1.10.0");
}

#[test]
fn prerelease_is_chosen_only_explicitly() {
    assert_eq!(pick("3.0.1-beta.1").unwrap(), "3.0.1-beta.1");
    assert_eq!(pick("*").unwrap(), "3.0.0");
    assert!(pick(">=3.0.1").is_err());
}

#[test]
fn tags_are_resolved_first() {
    let tags = BTreeMap::from([("latest".to_string(), "2.0.0".to_string())]);
    assert_eq!(resolve_version("latest", &versions(), &tags).unwrap(), "2.0.0");
    assert!(resolve_version("latest", &versions(), &BTreeMap::new()).is_err());
}

#[test]
fn unsupported_and_unsatisfiable_ranges_are_errors() {
    for spec in ["^9.0.0", "git+https://example.com/a.git", "npm:other@1", "file:../a", "a.b.c"] {
        assert!(pick(spec).is_err(), "{spec}");
    }
}
