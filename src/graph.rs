//! Граф зависимостей и его построение обходом в ширину (BFS).
//!
//! Вершина графа — пакет (в графе он один раз, даже если от него зависят
//! многие). Ребро `A → B` означает «пакет A напрямую зависит от пакета B».
//! Обход выполнен циклом с очередью, без рекурсии, поэтому глубокие цепочки
//! зависимостей не переполняют стек. Множество посещённых пакетов не даёт
//! зайти в пакет дважды, благодаря чему циклические зависимости
//! (`A → B → A`) не приводят к бесконечному обходу.

use std::collections::{BTreeSet, HashSet, VecDeque};

/// Прямая зависимость: имя пакета и требуемый диапазон версий.
pub type Requirement = (String, String);

/// Результат обращения к источнику данных об одном пакете.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Package {
    /// Выбранная версия. В тестовом репозитории версий нет — пустая строка.
    pub version: String,
    /// Прямые зависимости пакета.
    pub dependencies: Vec<Requirement>,
}

/// Источник сведений о пакетах: реальный репозиторий npm или тестовый файл.
pub trait DependencySource {
    /// Возвращает версию и прямые зависимости пакета `name`.
    ///
    /// `spec` — требуемая версия или диапазон версий (для корневого пакета —
    /// версия из конфигурации). Источники без версий игнорируют `spec`.
    ///
    /// # Ошибки
    ///
    /// Возвращает описание проблемы, если пакет не найден или недоступен.
    fn package(&mut self, name: &str, spec: &str) -> Result<Package, String>;
}

/// Вершина графа.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    /// Имя пакета.
    pub name: String,
    /// Выбранная версия (может быть пустой).
    pub version: String,
    /// Имена пакетов, от которых зависит этот пакет, без повторов,
    /// отсортированные по алфавиту.
    pub dependencies: Vec<String>,
}

/// Граф зависимостей. Вершины идут в порядке обхода в ширину,
/// первой стоит корневая вершина.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Graph {
    /// Все вершины графа в порядке обхода.
    pub nodes: Vec<Node>,
}

impl Node {
    /// Подпись вершины: `имя@версия`, а без версии — просто имя.
    pub fn label(&self) -> String {
        if self.version.is_empty() {
            self.name.clone()
        } else {
            format!("{}@{}", self.name, self.version)
        }
    }
}

impl Graph {
    /// Ищет вершину по имени пакета.
    pub fn node(&self, name: &str) -> Option<&Node> {
        self.nodes.iter().find(|node| node.name == name)
    }

    /// Количество рёбер (зависимостей) в графе.
    pub fn edge_count(&self) -> usize {
        self.nodes.iter().map(|node| node.dependencies.len()).sum()
    }
}

/// Строит граф зависимостей пакета `root` с учётом транзитивности.
///
/// Алгоритм (BFS): корневой пакет кладётся в очередь; пока очередь не пуста,
/// из неё берётся пакет, у источника запрашиваются его зависимости, и ещё не
/// встречавшиеся зависимости добавляются в конец очереди. Для каждого пакета
/// версия выбирается при первой встрече.
///
/// # Ошибки
///
/// Возвращает ошибку, если источник не смог выдать сведения о каком-либо
/// пакете. В сообщение добавляется имя этого пакета.
pub fn build_graph(
    source: &mut dyn DependencySource,
    root: &str,
    root_spec: &str,
) -> Result<Graph, String> {
    let mut graph = Graph::default();
    let mut visited: HashSet<String> = HashSet::from([root.to_string()]);
    let mut queue = VecDeque::from([(root.to_string(), root_spec.to_string())]);

    while let Some((name, spec)) = queue.pop_front() {
        let package = source.package(&name, &spec).map_err(|e| format!("пакет '{name}': {e}"))?;
        let mut dependencies = BTreeSet::new();
        for (dependency, range) in package.dependencies {
            dependencies.insert(dependency.clone());
            if visited.insert(dependency.clone()) {
                queue.push_back((dependency, range));
            }
        }
        graph.nodes.push(Node {
            name,
            version: package.version,
            dependencies: dependencies.into_iter().collect(),
        });
    }
    Ok(graph)
}

/// Форматирует граф для вывода на экран: заголовок и по строке
/// `пакет -> зависимость, зависимость` для каждой вершины.
pub fn format_graph(graph: &Graph) -> String {
    let root = graph.nodes.first().map(Node::label).unwrap_or_default();
    let mut text = format!(
        "Граф зависимостей {root}: пакетов {}, зависимостей {}\n",
        graph.nodes.len(),
        graph.edge_count()
    );
    for node in &graph.nodes {
        let targets = if node.dependencies.is_empty() {
            "(нет зависимостей)".to_string()
        } else {
            node.dependencies.join(", ")
        };
        text.push_str(&format!("  {} -> {targets}\n", node.label()));
    }
    text
}
