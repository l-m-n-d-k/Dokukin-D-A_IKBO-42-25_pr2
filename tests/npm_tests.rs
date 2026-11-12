//! Тесты разбора документов репозитория и построения адреса (без сети).

use depgraph::npm::{packument_url, parse_dependencies, parse_packument};

#[test]
fn url_is_built_from_repository_and_name() {
    let url = packument_url("https://registry.npmjs.org", "express");
    assert_eq!(url, "https://registry.npmjs.org/express");
}

#[test]
fn url_ignores_trailing_slash_and_keeps_scope() {
    let url = packument_url("http://localhost:4873/", "@babel/core");
    assert_eq!(url, "http://localhost:4873/@babel/core");
}

#[test]
fn dependencies_are_extracted_from_manifest() {
    let manifest = r#"{
        "name": "demo",
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
    assert!(parse_dependencies(r#"{"name": "leaf"}"#).unwrap().is_empty());
    assert!(parse_dependencies(r#"{"dependencies": {}}"#).unwrap().is_empty());
}

#[test]
fn broken_manifests_are_rejected() {
    for text in [
        "",
        "not json",
        "[]",
        r#"{"dependencies": []}"#,
        r#"{"dependencies": {"a": 1}}"#,
        r#"{"dependencies": {"a": null}}"#,
    ] {
        assert!(parse_dependencies(text).is_err(), "{text}");
    }
}

#[test]
fn packument_has_versions_and_tags() {
    let text = r#"{
        "name": "demo",
        "dist-tags": {"latest": "2.0.0"},
        "versions": {
            "1.0.0": {"dependencies": {"a": "^1.0.0"}},
            "2.0.0": {}
        }
    }"#;
    let packument = parse_packument(text).unwrap();
    assert_eq!(packument.dist_tags["latest"], "2.0.0");
    assert_eq!(packument.versions["1.0.0"]["a"], "^1.0.0");
    assert!(packument.versions["2.0.0"].is_empty());
}

#[test]
fn packument_without_tags_is_accepted() {
    let packument = parse_packument(r#"{"versions": {}}"#).unwrap();
    assert!(packument.dist_tags.is_empty());
    assert!(packument.versions.is_empty());
}

#[test]
fn broken_packuments_are_rejected() {
    for text in [
        "",
        "[]",
        "{}",
        r#"{"versions": []}"#,
        r#"{"versions": {"1.0.0": 5}}"#,
        r#"{"versions": {}, "dist-tags": []}"#,
        r#"{"versions": {}, "dist-tags": {"latest": 1}}"#,
        r#"{"versions": {"1.0.0": {"dependencies": {"a": 1}}}}"#,
    ] {
        assert!(parse_packument(text).is_err(), "{text}");
    }
}
