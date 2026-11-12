//! Тесты обратных зависимостей: уровни, циклы, ошибки, вывод.

use std::path::Path;

use depgraph::graph::{Graph, build_graph};
use depgraph::reverse::{Levels, format_reverse, reverse_dependencies};
use depgraph::test_repo::load_test_repository;

/// Строит граф тестового репозитория от пакета `root`.
fn graph_of(file: &str, root: &str) -> Graph {
    let path = Path::new("tests/fixtures/repos").join(file);
    let mut repository = load_test_repository(&path).unwrap();
    build_graph(&mut repository, root, "").unwrap()
}

/// Обратные зависимости пакета `target` в виде списка строк по уровням.
fn levels_of(file: &str, root: &str, target: &str) -> Vec<String> {
    let graph = graph_of(file, root);
    let levels: Levels = reverse_dependencies(&graph, target).unwrap();
    levels.iter().map(|level| level.join(",")).collect()
}

#[test]
fn direct_and_indirect_dependants_are_found_by_levels() {
    assert_eq!(levels_of("reverse_demo.txt", "A", "G"), ["D,E", "B,C", "A"]);
    assert_eq!(levels_of("reverse_demo.txt", "A", "E"), ["B,C", "A"]);
    assert_eq!(levels_of("reverse_demo.txt", "A", "F"), ["C", "A"]);
}

#[test]
fn chain_gives_one_package_per_level() {
    assert_eq!(levels_of("chain.txt", "A", "D"), ["C", "B", "A"]);
}

#[test]
fn shared_dependant_is_listed_once() {
    assert_eq!(levels_of("diamond.txt", "A", "D"), ["B,C", "A"]);
}

#[test]
fn root_has_no_reverse_dependencies() {
    assert!(levels_of("reverse_demo.txt", "A", "A").is_empty());
}

#[test]
fn cycle_terminates_and_excludes_the_package_itself() {
    assert_eq!(levels_of("cycle.txt", "A", "A"), ["C", "B"]);
    assert_eq!(levels_of("cycle.txt", "A", "C"), ["B", "A"]);
}

#[test]
fn several_cycles_terminate() {
    assert_eq!(levels_of("two_cycles.txt", "A", "D"), ["C", "B", "A"]);
}

#[test]
fn self_dependency_does_not_count() {
    assert_eq!(levels_of("self_cycle.txt", "A", "B"), ["A,C"]);
    assert!(levels_of("self_cycle.txt", "A", "A").is_empty());
}

#[test]
fn only_packages_of_the_graph_are_considered() {
    // Граф от B не содержит A и C, хотя они зависят от D в файле.
    assert_eq!(levels_of("diamond.txt", "B", "D"), ["B"]);
}

#[test]
fn package_outside_the_graph_is_an_error() {
    let graph = graph_of("reverse_demo.txt", "A");
    let error = reverse_dependencies(&graph, "Z").unwrap_err();
    assert!(error.contains("'Z'"), "{error}");
}

#[test]
fn result_is_formatted_by_levels() {
    let graph = graph_of("reverse_demo.txt", "A");
    let levels = reverse_dependencies(&graph, "G").unwrap();
    let expected = "Обратные зависимости G в графе A: 5\n\
                    \x20 уровень 1 (напрямую): D, E\n\
                    \x20 уровень 2 (через другие пакеты): B, C\n\
                    \x20 уровень 3 (через другие пакеты): A\n";
    assert_eq!(format_reverse(&graph, "G", &levels), expected);
}

#[test]
fn empty_result_is_formatted_with_a_note() {
    let graph = graph_of("reverse_demo.txt", "A");
    let text = format_reverse(&graph, "A", &Vec::new());
    assert!(text.starts_with("Обратные зависимости A в графе A: 0\n"));
    assert!(text.contains("ни один пакет"));
}
