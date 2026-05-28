//! VietCode Cortex — Bộ não hiểu codebase.
//!
//! Kiến trúc 3 tầng:
//! - Tầng 1: Symbol Index (tree-sitter parse → SQLite FTS5)
//! - Tầng 2: Relation Graph (call graph, dependency graph)
//! - Tầng 3: Semantic Understanding (embedding + summary - tương lai)

pub mod parser;
pub mod index;
pub mod graph;
pub mod watcher;

use serde::{Deserialize, Serialize};

/// Định danh duy nhất cho một symbol trong codebase.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SymbolId(pub String);

/// Loại symbol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SymbolKind {
    Function,
    Struct,
    Enum,
    Trait,
    Impl,
    Module,
    TypeAlias,
    Const,
    Static,
    Use,
    Macro,
}

impl SymbolKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            SymbolKind::Function => "fn",
            SymbolKind::Struct => "struct",
            SymbolKind::Enum => "enum",
            SymbolKind::Trait => "trait",
            SymbolKind::Impl => "impl",
            SymbolKind::Module => "mod",
            SymbolKind::TypeAlias => "type",
            SymbolKind::Const => "const",
            SymbolKind::Static => "static",
            SymbolKind::Use => "use",
            SymbolKind::Macro => "macro",
        }
    }
}

/// Một symbol được extract từ source code.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Symbol {
    pub id: SymbolId,
    pub name: String,
    pub kind: SymbolKind,
    pub file_path: String,
    pub line_start: usize,
    pub line_end: usize,
    pub column_start: usize,
    pub visibility: Visibility,
    /// Tên module cha (vd: "auth::handler")
    pub parent_module: Option<String>,
    /// Signature đầy đủ (vd: "fn authenticate(token: &str) -> Result<User>"")
    pub signature: Option<String>,
    /// Doc comment nếu có
    pub doc_comment: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Visibility {
    Public,
    Crate,
    Private,
}

/// Kết quả của việc index toàn bộ codebase.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexResult {
    pub files_processed: usize,
    pub symbols_found: usize,
    pub duration_ms: u64,
    pub errors: Vec<String>,
}
