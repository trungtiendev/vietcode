//! Relation Graph — Tầng 2 của Cortex.
//!
//! Xây dựng và query:
//! - Call graph: A gọi B
//! - Import graph: file X import module Y
//! - Type dependency: struct A có field kiểu B

use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::path::Path;

// ── Types ─────────────────────────────────────────────────────

/// Loại cạnh trong graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EdgeType {
    Calls,
    Contains,
    Imports,
}

impl EdgeType {
    fn as_str(&self) -> &'static str {
        match self {
            EdgeType::Calls => "calls",
            EdgeType::Contains => "contains",
            EdgeType::Imports => "imports",
        }
    }

    #[allow(dead_code)]
    fn from_str(s: &str) -> Self {
        match s {
            "calls" => EdgeType::Calls,
            "contains" => EdgeType::Contains,
            "imports" => EdgeType::Imports,
            _ => EdgeType::Calls,
        }
    }
}

/// Một cạnh trong graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub from_symbol: String,
    pub to_symbol: String,
    pub edge_type: EdgeType,
    pub file_path: String,
    pub line: usize,
}

/// Thông tin về một caller.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallerInfo {
    pub name: String,
    pub file_path: String,
    pub line: usize,
}

/// Thông tin về một callee.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalleeInfo {
    pub name: String,
    pub file_path: String,
    pub line: usize,
}

/// Kết quả phân tích impact.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactResult {
    pub symbol_name: String,
    pub callers: Vec<CallerInfo>,
    pub callees: Vec<CalleeInfo>,
}

/// Một bước trên đường đi ngắn nhất.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathStep {
    pub symbol: String,
    pub file_path: String,
}

// ── RelationGraph ─────────────────────────────────────────────

pub struct RelationGraph {
    conn: Connection,
}

impl RelationGraph {
    pub fn open(db_path: &Path) -> Result<Self> {
        let conn = Connection::open(db_path)
            .with_context(|| format!("Không thể mở database: {}", db_path.display()))?;
        let graph = Self { conn };
        graph.ensure_schema()?;
        Ok(graph)
    }

    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let graph = Self { conn };
        graph.ensure_schema()?;
        Ok(graph)
    }

    fn ensure_schema(&self) -> Result<()> {
        self.conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS edges (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                from_symbol TEXT NOT NULL,
                to_symbol   TEXT NOT NULL,
                edge_type   TEXT NOT NULL,
                file_path   TEXT NOT NULL,
                line        INTEGER NOT NULL DEFAULT 0
            );
            CREATE INDEX IF NOT EXISTS idx_edges_from ON edges(from_symbol);
            CREATE INDEX IF NOT EXISTS idx_edges_to   ON edges(to_symbol);
            CREATE INDEX IF NOT EXISTS idx_edges_type ON edges(edge_type);
            ",
        )?;
        Ok(())
    }

    /// Xóa edges cũ và insert mới.
    pub fn rebuild(&mut self, edges: &[Edge]) -> Result<usize> {
        let tx = self.conn.transaction()?;
        tx.execute("DELETE FROM edges", [])?;

        let mut count = 0;
        for edge in edges {
            tx.execute(
                "INSERT INTO edges (from_symbol, to_symbol, edge_type, file_path, line)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    edge.from_symbol,
                    edge.to_symbol,
                    edge.edge_type.as_str(),
                    edge.file_path,
                    edge.line as i64,
                ],
            )?;
            count += 1;
        }
        tx.commit()?;
        Ok(count)
    }

    pub fn callees(&self, symbol_name: &str) -> Result<Vec<CalleeInfo>> {
        let mut stmt = self.conn.prepare(
            "SELECT e.to_symbol, e.file_path, e.line
             FROM edges e WHERE e.from_symbol = ?1 AND e.edge_type = 'calls'
             ORDER BY e.to_symbol",
        )?;
        let rows = stmt.query_map(params![symbol_name], |row| {
            Ok(CalleeInfo { name: row.get(0)?, file_path: row.get(1)?, line: row.get::<_, i64>(2)? as usize })
        })?;
        let mut results = Vec::new();
        for row in rows { results.push(row?); }
        Ok(results)
    }

    pub fn callers(&self, symbol_name: &str) -> Result<Vec<CallerInfo>> {
        let mut stmt = self.conn.prepare(
            "SELECT e.from_symbol, e.file_path, e.line
             FROM edges e WHERE e.to_symbol = ?1 AND e.edge_type = 'calls'
             ORDER BY e.from_symbol",
        )?;
        let rows = stmt.query_map(params![symbol_name], |row| {
            Ok(CallerInfo { name: row.get(0)?, file_path: row.get(1)?, line: row.get::<_, i64>(2)? as usize })
        })?;
        let mut results = Vec::new();
        for row in rows { results.push(row?); }
        Ok(results)
    }

    pub fn impact(&self, symbol_name: &str) -> Result<ImpactResult> {
        Ok(ImpactResult {
            symbol_name: symbol_name.to_string(),
            callers: self.callers(symbol_name)?,
            callees: self.callees(symbol_name)?,
        })
    }

    pub fn shortest_path(&self, from: &str, to: &str) -> Result<Option<Vec<PathStep>>> {
        let mut stmt = self.conn.prepare(
            "SELECT to_symbol FROM edges WHERE from_symbol = ?1 AND edge_type = 'calls'",
        )?;

        let mut visited: HashMap<String, Option<String>> = HashMap::new();
        let mut queue: VecDeque<String> = VecDeque::new();
        visited.insert(from.to_string(), None);
        queue.push_back(from.to_string());

        let mut found = false;
        while let Some(current) = queue.pop_front() {
            if current == to { found = true; break; }
            let neighbors = stmt.query_map(params![current], |row| row.get::<_, String>(0))?;
            for neighbor in neighbors {
                let neighbor = neighbor?;
                if !visited.contains_key(&neighbor) {
                    visited.insert(neighbor.clone(), Some(current.clone()));
                    queue.push_back(neighbor);
                }
            }
        }

        if !found { return Ok(None); }

        let mut path = Vec::new();
        let mut current = to.to_string();
        let file = self.symbol_file(&current).unwrap_or_default();
        path.push(PathStep { symbol: current.clone(), file_path: file });

        while let Some(Some(prev)) = visited.get(&current).cloned() {
            let file = self.symbol_file(&prev).unwrap_or_default();
            path.push(PathStep { symbol: prev.clone(), file_path: file });
            current = prev;
        }
        path.reverse();
        Ok(Some(path))
    }

    fn symbol_file(&self, name: &str) -> Result<String> {
        self.conn.query_row(
            "SELECT file_path FROM symbols WHERE name = ?1 LIMIT 1",
            params![name], |row| row.get(0),
        ).or_else(|_| Ok("unknown".to_string()))
    }

    pub fn total_edges(&self) -> Result<usize> {
        let count: usize = self.conn.query_row("SELECT COUNT(*) FROM edges", [], |row| row.get(0))?;
        Ok(count)
    }

    pub fn count_by_type(&self) -> Result<Vec<(String, usize)>> {
        let mut stmt = self.conn.prepare(
            "SELECT edge_type, COUNT(*) FROM edges GROUP BY edge_type ORDER BY COUNT(*) DESC",
        )?;
        let rows = stmt.query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, usize>(1)?)))?;
        let mut results = Vec::new();
        for row in rows { results.push(row?); }
        Ok(results)
    }
}

// ── Edge Extraction ───────────────────────────────────────────

use crate::parser::Language;

pub fn extract_edges(
    source: &str,
    lang: Language,
    file_path: &str,
    known_symbols: &HashMap<String, crate::SymbolId>,
) -> Vec<Edge> {
    match lang {
        Language::Rust => extract_rust_edges(source, file_path, known_symbols),
    }
}

fn get_call_name(node: &tree_sitter::Node, source: &str) -> Option<String> {
    let func_node = node.child_by_field_name("function")?;
    let text = func_node.utf8_text(source.as_bytes()).ok()?;
    if func_node.kind() == "field_expression" {
        if let Some(field) = func_node.child_by_field_name("field") {
            return field.utf8_text(source.as_bytes()).ok().map(|s| s.to_string());
        }
    }
    Some(text.to_string())
}

fn extract_rust_edges(source: &str, file_path: &str, known_symbols: &HashMap<String, crate::SymbolId>) -> Vec<Edge> {
    let mut parser = tree_sitter::Parser::new();
    if parser.set_language(&tree_sitter_rust::LANGUAGE.into()).is_err() {
        return vec![];
    }
    let tree = match parser.parse(source, None) {
        Some(t) => t,
        None => return vec![],
    };
    let mut edges = Vec::new();
    walk_for_edges(tree.root_node(), source, file_path, known_symbols, &mut edges);
    edges
}

fn walk_for_edges(
    node: tree_sitter::Node,
    source: &str,
    file_path: &str,
    known_symbols: &HashMap<String, crate::SymbolId>,
    edges: &mut Vec<Edge>,
) {
    // Call edges
    if node.kind() == "call_expression" {
        if let Some(call_name) = get_call_name(&node, source) {
            if known_symbols.contains_key(&call_name) {
                let start = node.start_position();
                edges.push(Edge {
                    from_symbol: String::new(),
                    to_symbol: call_name,
                    edge_type: EdgeType::Calls,
                    file_path: file_path.to_string(),
                    line: start.row + 1,
                });
            }
        }
    }

    // Type edges (struct fields)
    if node.kind() == "field_declaration" {
        if let Some(type_node) = node.child_by_field_name("type") {
            let type_text = type_node.utf8_text(source.as_bytes()).unwrap_or("").to_string();
            let type_name = type_text
                .replace("Vec<", "").replace("Option<", "").replace("HashMap<", "")
                .replace("&", "").replace("&mut ", "").replace("Box<", "")
                .replace("Arc<", "").replace("Rc<", "")
                .trim_end_matches('>').trim().to_string();
            if known_symbols.contains_key(&type_name) {
                let start = node.start_position();
                edges.push(Edge {
                    from_symbol: String::new(),
                    to_symbol: type_name,
                    edge_type: EdgeType::Contains,
                    file_path: file_path.to_string(),
                    line: start.row + 1,
                });
            }
        }
    }

    // Import edges
    if node.kind() == "use_declaration" {
        let use_text = node.utf8_text(source.as_bytes()).unwrap_or("").to_string();
        let use_text = use_text.trim_start_matches("use ").trim_end_matches(';').trim().to_string();
        let start = node.start_position();
        edges.push(Edge {
            from_symbol: String::new(),
            to_symbol: use_text,
            edge_type: EdgeType::Imports,
            file_path: file_path.to_string(),
            line: start.row + 1,
        });
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        walk_for_edges(child, source, file_path, known_symbols, edges);
    }
}

/// Gán `from_symbol` cho mỗi edge dựa trên function cha chứa nó.
pub fn annotate_edges_with_parent(source: &str, lang: Language, file_path: &str, edges: &mut [Edge]) {
    match lang {
        Language::Rust => annotate_rust_edges(source, file_path, edges),
    }
}

struct FnInfo {
    name: String,
    start_byte: usize,
    end_byte: usize,
}

fn annotate_rust_edges(source: &str, file_path: &str, edges: &mut [Edge]) {
    let mut parser = tree_sitter::Parser::new();
    if parser.set_language(&tree_sitter_rust::LANGUAGE.into()).is_err() { return; }
    let tree = match parser.parse(source, None) {
        Some(t) => t,
        None => return,
    };

    let mut functions = Vec::new();
    collect_functions(tree.root_node(), source, &mut functions);

    for edge in edges.iter_mut() {
        if !edge.from_symbol.is_empty() { continue; }
        if let Some(edge_byte) = line_to_byte_offset(source, edge.line) {
            for func in &functions {
                if edge_byte >= func.start_byte && edge_byte <= func.end_byte {
                    edge.from_symbol = func.name.clone();
                    break;
                }
            }
        }
        if edge.from_symbol.is_empty() {
            edge.from_symbol = file_path.to_string();
        }
    }
}

fn collect_functions(node: tree_sitter::Node, source: &str, funcs: &mut Vec<FnInfo>) {
    if node.kind() == "function_item" {
        let start = node.start_byte();
        let end = node.end_byte();
        let name = node.child_by_field_name("name")
            .and_then(|n| n.utf8_text(source.as_bytes()).ok())
            .unwrap_or("unknown").to_string();
        funcs.push(FnInfo { name, start_byte: start, end_byte: end });
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_functions(child, source, funcs);
    }
}

fn line_to_byte_offset(source: &str, line: usize) -> Option<usize> {
    let mut current_line = 1;
    for (i, c) in source.char_indices() {
        if current_line == line { return Some(i); }
        if c == '\n' { current_line += 1; }
    }
    None
}

// ── Build graph từ thư mục ────────────────────────────────────

pub fn build_graph(
    source_dir: &Path,
    db_path: &Path,
    symbol_names: &HashMap<String, crate::SymbolId>,
) -> Result<(usize, Vec<String>)> {
    let mut files = Vec::new();
    crate::index::collect_rust_files(source_dir, &mut files)?;

    let mut all_edges = Vec::new();
    let mut errors = Vec::new();

    for file_path in &files {
        match std::fs::read_to_string(file_path) {
            Ok(source) => {
                let lang = Language::from_extension(file_path).unwrap_or(Language::Rust);
                let fp = file_path.to_string_lossy().to_string();
                let mut edges = extract_edges(&source, lang, &fp, symbol_names);
                annotate_edges_with_parent(&source, lang, &fp, &mut edges);
                all_edges.extend(edges);
            }
            Err(e) => errors.push(format!("{}: {}", file_path.display(), e)),
        }
    }

    let edge_count = all_edges.len();
    let mut graph = RelationGraph::open(db_path)?;
    graph.rebuild(&all_edges)?;
    Ok((edge_count, errors))
}

// ── Tests ─────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_symbols(names: &[&str]) -> HashMap<String, crate::SymbolId> {
        names.iter().map(|n| (n.to_string(), crate::SymbolId(n.to_string()))).collect()
    }

    #[test]
    fn extract_call_edges() {
        let source = r#"
fn authenticate() -> bool { true }
pub fn login() { authenticate(); validate(); }
fn validate() -> bool { true }
"#;
        let syms = make_symbols(&["authenticate", "login", "validate"]);
        let edges = extract_edges(source, Language::Rust, "test.rs", &syms);
        let calls: Vec<_> = edges.iter().filter(|e| e.edge_type == EdgeType::Calls).collect();
        assert!(!calls.is_empty(), "Expected call edges, got 0");
    }

    #[test]
    fn graph_build_and_query() {
        let mut graph = RelationGraph::open_in_memory().unwrap();
        let edges = vec![
            Edge { from_symbol: "main".into(), to_symbol: "login".into(), edge_type: EdgeType::Calls, file_path: "src/main.rs".into(), line: 10 },
            Edge { from_symbol: "login".into(), to_symbol: "authenticate".into(), edge_type: EdgeType::Calls, file_path: "src/auth.rs".into(), line: 20 },
        ];
        graph.rebuild(&edges).unwrap();

        assert_eq!(graph.callers("authenticate").unwrap().len(), 1);
        assert_eq!(graph.callees("main").unwrap().len(), 1);

        let impact = graph.impact("login").unwrap();
        assert_eq!(impact.callers.len(), 1);
        assert_eq!(impact.callees.len(), 1);
    }

    #[test]
    fn shortest_path() {
        let mut graph = RelationGraph::open_in_memory().unwrap();
        let edges = vec![
            Edge { from_symbol: "A".into(), to_symbol: "B".into(), edge_type: EdgeType::Calls, file_path: "x.rs".into(), line: 1 },
            Edge { from_symbol: "B".into(), to_symbol: "C".into(), edge_type: EdgeType::Calls, file_path: "x.rs".into(), line: 2 },
            Edge { from_symbol: "C".into(), to_symbol: "D".into(), edge_type: EdgeType::Calls, file_path: "x.rs".into(), line: 3 },
        ];
        graph.rebuild(&edges).unwrap();
        let path = graph.shortest_path("A", "D").unwrap();
        assert!(path.is_some());
        let path = path.unwrap();
        assert_eq!(path.len(), 4);
    }
}
