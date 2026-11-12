//! Тесты построения графа: обход в ширину, циклы, форматирование.

use std::path::Path;

use depgraph::graph::{Graph, build_graph, format_graph};
use depgraph::test_repo::{TestRepository, load_test_repository};

/// Нулевое значение (для разбора числа по буквам).
const ZERO: usize = 0;

/// Загружает тестовый репозиторий из `tests/fixtures/repos`.
fn repo(name: &str) -> TestRepository {
    load_test_repository(&Path::new("tests/fixtures/repos").join(name)).unwrap()
}

/// Строит граф тестового репозитория от пакета `root`.
fn graph_of(file: &str, root: &str) -> Result<Graph, String> {
    build_graph(&mut repo(file), root, "")
}

/// Имена вершин в порядке обхода.
fn order(graph: &Graph) -> Vec<&str> {
    graph.nodes.iter().map(|n| n.name.as_str()).collect()
}

/// Зависимости вершины.
fn deps<'a>(graph: &'a Graph, name: &str) -> Vec<&'a str> {
    graph.node(name).unwrap().dependencies.iter().map(String::as_str).collect()
}

#[test]
fn transitive_dependencies_are_included() {
    let graph = graph_of("chain.txt", "A").unwrap();
    assert_eq!(order(&graph), ["A", "B", "C", "D"]);
    assert_eq!(deps(&graph, "C"), ["D"]);
}

#[test]
fn traversal_is_breadth_first() {
    let graph = graph_of("tree.txt", "A").unwrap();
    assert_eq!(order(&graph), ["A", "B", "C", "D", "E", "F", "G"]);
}

#[test]
fn shared_dependency_appears_once() {
    let graph = graph_of("diamond.txt", "A").unwrap();
    assert_eq!(order(&graph), ["A", "B", "C", "D"]);
    assert_eq!(graph.edge_count(), 4);
}

#[test]
fn simple_cycle_terminates() {
    let graph = graph_of("cycle.txt", "A").unwrap();
    assert_eq!(order(&graph), ["A", "B", "C"]);
    assert_eq!(deps(&graph, "C"), ["A"]);
}

#[test]
fn self_dependency_and_mutual_cycle_terminate() {
    let graph = graph_of("self_cycle.txt", "A").unwrap();
    assert_eq!(order(&graph), ["A", "B", "C"]);
    assert_eq!(deps(&graph, "A"), ["A", "B"]);
    assert_eq!(deps(&graph, "B"), ["C"]);
    assert_eq!(deps(&graph, "C"), ["B"]);
}

#[test]
fn several_cycles_terminate() {
    let graph = graph_of("two_cycles.txt", "A").unwrap();
    assert_eq!(order(&graph), ["A", "B", "E", "C", "D"]);
    assert_eq!(deps(&graph, "C"), ["A", "D"]);
    assert_eq!(deps(&graph, "D"), ["B"]);
}

#[test]
fn graph_can_start_from_any_package() {
    let graph = graph_of("diamond.txt", "B").unwrap();
    assert_eq!(order(&graph), ["B", "D"]);
}

#[test]
fn packages_without_description_are_leaves() {
    let graph = graph_of("undeclared.txt", "A").unwrap();
    assert_eq!(order(&graph), ["A", "Y", "Z"]);
    assert!(deps(&graph, "Y").is_empty());
}

#[test]
fn unknown_root_is_an_error_with_package_name() {
    let error = graph_of("chain.txt", "Q").unwrap_err();
    assert!(error.contains("'Q'"), "{error}");
}

#[test]
fn very_long_chain_does_not_overflow_the_stack() {
    let size = 50_000;
    let mut text = String::new();
    for i in 0..size {
        text.push_str(&format!("P{}: P{}\n", letters(i), letters(i + 1)));
    }
    let mut repository = depgraph::test_repo::parse_test_repository(&text).unwrap();
    let graph = build_graph(&mut repository, &format!("P{}", letters(0)), "").unwrap();
    assert_eq!(graph.nodes.len(), size + 1);
}

/// Количество букв в латинском алфавите.
const ALPHABET: usize = 26;

/// Записывает число большими латинскими буквами (A = 0, B = 1, ...).
fn letters(number: usize) -> String {
    let letter = char::from(b'A' + (number % ALPHABET) as u8);
    match number / ALPHABET {
        ZERO => letter.to_string(),
        rest => format!("{}{letter}", letters(rest - 1)),
    }
}

#[test]
fn graph_is_formatted_one_package_per_line() {
    let graph = graph_of("diamond.txt", "A").unwrap();
    let expected = "Граф зависимостей A: пакетов 4, зависимостей 4\n\
                    \x20 A -> B, C\n\
                    \x20 B -> D\n\
                    \x20 C -> D\n\
                    \x20 D -> (нет зависимостей)\n";
    assert_eq!(format_graph(&graph), expected);
}
