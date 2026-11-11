//! Тесты проверок отдельных параметров конфигурации.

use depgraph::validate::{
    MAX_NPM_NAME_LEN, check_npm_package_name, check_output_file, check_package_name,
    check_repository, check_test_package_name, check_url, check_version,
};

#[test]
fn npm_names_accepted() {
    for name in ["express", "lodash.merge", "@types/node", "a-b_c~d", "react-dom"] {
        assert!(check_npm_package_name(name).is_ok(), "{name}");
    }
}

#[test]
fn npm_names_rejected() {
    let too_long = "a".repeat(MAX_NPM_NAME_LEN + 1);
    for name in ["Express", ".hidden", "_private", "has space", "@scope", "@/x", "a/b!", &too_long]
    {
        assert!(check_npm_package_name(name).is_err(), "{name}");
    }
}

#[test]
fn empty_package_name_rejected_in_both_modes() {
    assert!(check_package_name("", false).is_err());
    assert!(check_package_name("", true).is_err());
}

#[test]
fn test_mode_names_must_be_uppercase_latin() {
    assert!(check_test_package_name("A").is_ok());
    assert!(check_test_package_name("ABC").is_ok());
    assert!(check_test_package_name("a").is_err());
    assert!(check_test_package_name("A1").is_err());
    assert!(check_test_package_name("Ж").is_err());
}

#[test]
fn urls() {
    assert!(check_url("https://registry.npmjs.org").is_ok());
    assert!(check_url("http://localhost:4873/").is_ok());
    assert!(check_url("registry.npmjs.org").is_err());
    assert!(check_url("ftp://example.com").is_err());
    assert!(check_url("https://").is_err());
    assert!(check_url("https://exa mple.com").is_err());
}

#[test]
fn repository_depends_on_mode() {
    let file = "tests/fixtures/test_repo.txt";
    assert!(check_repository(file, true).is_ok());
    assert!(check_repository(file, false).is_err());
    assert!(check_repository("https://registry.npmjs.org", true).is_err());
    assert!(check_repository("tests/fixtures", true).is_err());
    assert!(check_repository("", false).is_err());
}

#[test]
fn versions_accepted() {
    for v in ["0.0.0", "4.18.2", "10.20.30", "1.0.0-beta.1", "1.0.0+build.5", "1.0.0-rc-1+sha"] {
        assert!(check_version(v).is_ok(), "{v}");
    }
}

#[test]
fn versions_rejected() {
    let bad = [
        "", "1", "1.2", "1.2.3.4", "01.2.3", "1.x.3", "v1.2.3", "1.2.3-", "1.2.3+", "1.2.3-a..b",
    ];
    for v in bad {
        assert!(check_version(v).is_err(), "{v}");
    }
}

#[test]
fn output_files() {
    assert!(check_output_file("graph.png").is_ok());
    assert!(check_output_file("GRAPH.PNG").is_ok());
    assert!(check_output_file("tests/graph.png").is_ok());
    assert!(check_output_file("").is_err());
    assert!(check_output_file("graph.jpg").is_err());
    assert!(check_output_file("graph").is_err());
    assert!(check_output_file("no_such_dir/graph.png").is_err());
}
