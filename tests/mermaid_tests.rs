//! Тесты формирования текста Mermaid.

use std::path::Path;

use depgraph::graph::{Graph, Node, build_graph};
use depgraph::mermaid::to_mermaid;
use depgraph::test_repo::load_test_repository;

/// Строит граф тестового репозитория от пакета `A`.
fn graph_of(file: &str) -> Graph {
    let path = Path::new("tests/fixtures/repos").join(file);
    let mut repository = load_test_repository(&path).unwrap();
    build_graph(&mut repository, "A", "").unwrap()
}

#[test]
fn diamond_is_described_by_nodes_and_edges() {
    let expected = "graph TD\n\
                    \x20   n0[\"A\"]\n\
                    \x20   n1[\"B\"]\n\
                    \x20   n2[\"C\"]\n\
                    \x20   n3[\"D\"]\n\
                    \x20   n0 --> n1\n\
                    \x20   n0 --> n2\n\
                    \x20   n1 --> n3\n\
                    \x20   n2 --> n3\n";
    assert_eq!(to_mermaid(&graph_of("diamond.txt")), expected);
}

#[test]
fn cycle_becomes_an_edge_back_to_an_earlier_node() {
    let text = to_mermaid(&graph_of("cycle.txt"));
    assert!(text.contains("n2 --> n0"), "{text}");
}

#[test]
fn self_dependency_becomes_a_loop() {
    let text = to_mermaid(&graph_of("self_cycle.txt"));
    assert!(text.contains("n0 --> n0"), "{text}");
}

#[test]
fn package_without_dependencies_has_no_edges() {
    let graph = Graph {
        nodes: vec![Node { name: "lone".into(), version: "1.0.0".into(), dependencies: vec![] }],
    };
    assert_eq!(to_mermaid(&graph), "graph TD\n    n0[\"lone@1.0.0\"]\n");
}

#[test]
fn labels_keep_scope_and_version_and_ids_stay_simple() {
    let graph = Graph {
        nodes: vec![
            Node {
                name: "@babel/core".into(),
                version: "7.0.0".into(),
                dependencies: vec!["lodash.merge".into()],
            },
            Node { name: "lodash.merge".into(), version: "4.6.2".into(), dependencies: vec![] },
        ],
    };
    let text = to_mermaid(&graph);
    assert!(text.contains("n0[\"@babel/core@7.0.0\"]"), "{text}");
    assert!(text.contains("n1[\"lodash.merge@4.6.2\"]"), "{text}");
    assert!(text.contains("n0 --> n1"), "{text}");
}

#[test]
fn quotes_in_labels_are_escaped() {
    let graph = Graph {
        nodes: vec![Node { name: "a\"b".into(), version: String::new(), dependencies: vec![] }],
    };
    assert!(to_mermaid(&graph).contains("n0[\"a#quot;b\"]"));
}

#[test]
fn empty_graph_gives_only_header() {
    assert_eq!(to_mermaid(&Graph::default()), "graph TD\n");
}
