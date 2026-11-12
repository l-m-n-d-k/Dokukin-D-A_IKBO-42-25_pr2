//! Тесты разбора манифеста и построения адреса (без обращения к сети).

use depgraph::npm::{Dependencies, format_dependencies, manifest_url, parse_dependencies};

#[test]
fn url_is_built_from_repository_name_and_version() {
    let url = manifest_url("https://registry.npmjs.org", "express", "4.18.2");
    assert_eq!(url, "https://registry.npmjs.org/express/4.18.2");
}

#[test]
fn url_ignores_trailing_slash_and_keeps_scope() {
    let url = manifest_url("http://localhost:4873/", "@babel/core", "7.0.0");
    assert_eq!(url, "http://localhost:4873/@babel/core/7.0.0");
}

#[test]
fn dependencies_are_extracted() {
    let manifest = r#"{
        "name": "demo",
        "version": "1.0.0",
        "dependencies": {"zlib": "^1.0.0", "accepts": "~1.3.8"},
        "devDependencies": {"mocha": "10.0.0"}
    }"#;
    let deps = parse_dependencies(manifest).unwrap();
    let names: Vec<_> = deps.keys().map(String::as_str).collect();
    assert_eq!(names, ["accepts", "zlib"]);
    assert_eq!(deps["accepts"], "~1.3.8");
}

#[test]
fn missing_dependencies_field_means_no_dependencies() {
    let deps = parse_dependencies(r#"{"name": "leaf", "version": "1.0.0"}"#).unwrap();
    assert!(deps.is_empty());
}

#[test]
fn empty_dependencies_object_means_no_dependencies() {
    assert!(parse_dependencies(r#"{"dependencies": {}}"#).unwrap().is_empty());
}

#[test]
fn broken_manifests_are_rejected() {
    for text in [
        "",
        "not json",
        "[]",
        "42",
        r#"{"dependencies": []}"#,
        r#"{"dependencies": "a"}"#,
        r#"{"dependencies": {"a": 1}}"#,
        r#"{"dependencies": {"a": null}}"#,
    ] {
        assert!(parse_dependencies(text).is_err(), "{text}");
    }
}

#[test]
fn dependencies_are_formatted_one_per_line() {
    let deps = Dependencies::from([
        ("b".to_string(), "^2.0.0".to_string()),
        ("a".to_string(), "1.0.0".to_string()),
    ]);
    let expected = "Прямые зависимости demo@1.0.0: 2\n  a: 1.0.0\n  b: ^2.0.0\n";
    assert_eq!(format_dependencies("demo", "1.0.0", &deps), expected);
}

#[test]
fn empty_dependencies_are_formatted_with_a_note() {
    let text = format_dependencies("leaf", "1.0.0", &Dependencies::new());
    assert_eq!(text, "Прямые зависимости leaf@1.0.0: 0\n  (нет прямых зависимостей)\n");
}
