//! Тесты чтения файла тестового репозитория.

use std::path::Path;

use depgraph::graph::DependencySource;
use depgraph::test_repo::{load_test_repository, parse_test_repository};

#[test]
fn packages_and_dependencies_are_read() {
    let mut repo = parse_test_repository("A: B C\nB: C\nC:\n").unwrap();
    let a = repo.package("A", "").unwrap();
    let names: Vec<_> = a.dependencies.iter().map(|(n, _)| n.as_str()).collect();
    assert_eq!(names, ["B", "C"]);
    assert!(a.version.is_empty());
    assert!(repo.package("C", "").unwrap().dependencies.is_empty());
}

#[test]
fn comments_and_blank_lines_are_skipped() {
    let text = "# заголовок\n\nA: B # хвостовой комментарий\n   \nB:\n";
    let mut repo = parse_test_repository(text).unwrap();
    assert_eq!(repo.package("A", "").unwrap().dependencies.len(), 1);
}

#[test]
fn package_mentioned_only_as_dependency_is_a_leaf() {
    let mut repo = parse_test_repository("A: Z\n").unwrap();
    assert!(repo.package("Z", "").unwrap().dependencies.is_empty());
}

#[test]
fn unknown_package_is_an_error() {
    let mut repo = parse_test_repository("A: B\n").unwrap();
    assert!(repo.package("Q", "").is_err());
}

#[test]
fn empty_file_is_an_empty_repository() {
    let mut repo = parse_test_repository("").unwrap();
    assert!(repo.package("A", "").is_err());
}

#[test]
fn broken_files_are_rejected_with_line_numbers() {
    let cases = [
        ("A: B\nsome text\n", "строка 2"),
        ("A: b\n", "строка 1"),
        ("A: B1\n", "строка 1"),
        ("a: B\n", "строка 1"),
        ("A: B\nA: C\n", "повторно"),
        (": B\n", "не указано имя"),
        ("\n\nA B\n", "строка 3"),
    ];
    for (text, expected) in cases {
        let error = parse_test_repository(text).unwrap_err();
        assert!(error.contains(expected), "{text:?}: {error}");
    }
}

#[test]
fn file_is_loaded_from_disk() {
    let repo = load_test_repository(Path::new("tests/fixtures/repos/diamond.txt"));
    assert!(repo.is_ok());
}

#[test]
fn missing_file_is_an_error() {
    let error = load_test_repository(Path::new("tests/fixtures/repos/none.txt")).unwrap_err();
    assert!(error.contains("не удалось прочитать"), "{error}");
}
