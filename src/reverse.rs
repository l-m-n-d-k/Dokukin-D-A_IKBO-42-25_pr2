//! Обратные зависимости: какие пакеты графа зависят от заданного пакета.
//!
//! Если в графе есть ребро `A -> B` («A зависит от B»), то для пакета `B`
//! пакет `A` — его обратная зависимость. Обратные зависимости ищутся по всему
//! графу транзитивно: если `A -> B -> C`, то для `C` обратными зависимостями
//! будут и `B` (напрямую), и `A` (через `B`).
//!
//! Используется тот же обход в ширину, что и при построении графа, но по
//! перевёрнутым рёбрам. Результат разбит на уровни: на первом уровне пакеты,
//! зависящие от заданного напрямую, на втором — зависящие от них и т. д.

use std::collections::{BTreeMap, HashSet, VecDeque};

use crate::graph::Graph;

/// Номер первого уровня обратных зависимостей (зависимость напрямую).
const DIRECT_LEVEL: usize = 0;

/// Обратные зависимости по уровням: `levels[0]` — зависящие напрямую,
/// `levels[1]` — зависящие через один пакет и т. д. Внутри уровня пакеты
/// отсортированы по алфавиту.
pub type Levels = Vec<Vec<String>>;

/// Строит перевёрнутые рёбра: пакет → пакеты, которые напрямую от него зависят.
fn reverse_edges(graph: &Graph) -> BTreeMap<&str, Vec<&str>> {
    let mut edges: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for node in &graph.nodes {
        for dependency in &node.dependencies {
            edges.entry(dependency.as_str()).or_default().push(node.name.as_str());
        }
    }
    edges
}

/// Находит все пакеты графа, которые зависят от `target` напрямую или
/// через другие пакеты.
///
/// Сам `target` в результат не попадает, даже если он входит в цикл.
/// Повторно один пакет не посещается, поэтому циклы обход не зацикливают.
///
/// # Ошибки
///
/// Возвращает описание проблемы, если пакета `target` нет в графе.
pub fn reverse_dependencies(graph: &Graph, target: &str) -> Result<Levels, String> {
    if graph.node(target).is_none() {
        return Err(format!("пакет '{target}' не входит в граф зависимостей"));
    }
    let edges = reverse_edges(graph);
    let mut visited: HashSet<&str> = HashSet::from([target]);
    let mut queue = VecDeque::from([(target, 0)]);
    let mut levels: Levels = Vec::new();

    while let Some((name, depth)) = queue.pop_front() {
        let dependants = edges.get(name).map(Vec::as_slice).unwrap_or_default();
        for &dependant in dependants {
            if !visited.insert(dependant) {
                continue;
            }
            if levels.len() == depth {
                levels.push(Vec::new());
            }
            levels[depth].push(dependant.to_string());
            queue.push_back((dependant, depth + 1));
        }
    }
    levels.iter_mut().for_each(|level| level.sort());
    Ok(levels)
}

/// Форматирует обратные зависимости для вывода на экран: заголовок и
/// по строке на каждый уровень.
pub fn format_reverse(graph: &Graph, target: &str, levels: &Levels) -> String {
    let root = graph.nodes.first().map(|node| node.label()).unwrap_or_default();
    let total: usize = levels.iter().map(Vec::len).sum();
    let mut text = format!("Обратные зависимости {target} в графе {root}: {total}\n");
    if levels.is_empty() {
        text.push_str("  (ни один пакет графа не зависит от этого пакета)\n");
    }
    for (index, level) in levels.iter().enumerate() {
        let kind = if index == DIRECT_LEVEL { "напрямую" } else { "через другие пакеты" };
        text.push_str(&format!("  уровень {} ({kind}): {}\n", index + 1, level.join(", ")));
    }
    text
}
