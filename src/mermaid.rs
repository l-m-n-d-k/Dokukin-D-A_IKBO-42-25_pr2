//! Текстовое представление графа зависимостей на языке диаграмм Mermaid.
//!
//! Результат — схема `graph TD` (сверху вниз): сначала описания вершин, затем
//! рёбра. Идентификаторы вершин в Mermaid не могут содержать `@`, `/`, `.` и
//! другие символы из имён npm-пакетов, поэтому вершинам даются служебные
//! идентификаторы `n0`, `n1`, ..., а настоящее имя пакета записывается в
//! подписи в кавычках.
//!
//! ```text
//! graph TD
//!     n0["A@1.0.0"]
//!     n1["B@2.0.0"]
//!     n0 --> n1
//! ```

use std::collections::HashMap;

use crate::graph::Graph;

/// Заголовок схемы: граф, направленный сверху вниз.
const HEADER: &str = "graph TD";

/// Отступ строк внутри схемы.
const INDENT: &str = "    ";

/// Экранирует подпись вершины: кавычка внутри подписи ломает синтаксис Mermaid.
fn escape_label(label: &str) -> String {
    label.replace('"', "#quot;")
}

/// Служебный идентификатор вершины по её номеру.
fn node_id(index: usize) -> String {
    format!("n{index}")
}

/// Формирует Mermaid-текст графа.
///
/// Вершины идут в порядке обхода в ширину, рёбра — в том же порядке, а внутри
/// вершины — по алфавиту. Циклические зависимости превращаются в обычные
/// рёбра, которые возвращаются к уже нарисованным вершинам.
pub fn to_mermaid(graph: &Graph) -> String {
    let ids: HashMap<&str, usize> =
        graph.nodes.iter().enumerate().map(|(index, node)| (node.name.as_str(), index)).collect();

    let mut lines = vec![HEADER.to_string()];
    for (index, node) in graph.nodes.iter().enumerate() {
        lines.push(format!("{INDENT}{}[\"{}\"]", node_id(index), escape_label(&node.label())));
    }
    for (index, node) in graph.nodes.iter().enumerate() {
        for dependency in &node.dependencies {
            if let Some(&target) = ids.get(dependency.as_str()) {
                lines.push(format!("{INDENT}{} --> {}", node_id(index), node_id(target)));
            }
        }
    }
    lines.join("\n") + "\n"
}
