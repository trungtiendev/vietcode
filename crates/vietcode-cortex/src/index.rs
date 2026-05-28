//! SQLite Symbol Index — Tầng 1 của Cortex.
//!
//! Lưu trữ tất cả symbols trong SQLite với FTS5 full-text search.
//! Cho phép query nhanh: tìm kiếm theo tên, kind, file, module.

use crate::{IndexResult, Symbol, SymbolKind};
use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use std::path::Path;
use std::time::Instant;

/// Quản lý SQLite database cho symbol index.
pub struct SymbolIndex {
    conn: Connection,
}

impl SymbolIndex {
    /// Mở hoặc tạo database tại `db_path`.
    /// Database được tạo tự động nếu chưa tồn tại.
    pub fn open(db_path: &Path) -> Result<Self> {
        let conn = Connection::open(db_path)
            .with_context(|| format!("Không thể mở database: {}", db_path.display()))?;

        let mut index = Self { conn };
        index.ensure_schema()?;
        Ok(index)
    }

    /// Tạo database trong memory (cho testing).
    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let mut index = Self { conn };
        index.ensure_schema()?;
        Ok(index)
    }

    /// Đảm bảo schema đã được tạo.
    fn ensure_schema(&mut self) -> Result<()> {
        self.conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS symbols (
                id          TEXT PRIMARY KEY,
                name        TEXT NOT NULL,
                kind        TEXT NOT NULL,
                file_path   TEXT NOT NULL,
                line_start  INTEGER NOT NULL,
                line_end    INTEGER NOT NULL,
                column_start INTEGER NOT NULL DEFAULT 0,
                visibility  TEXT NOT NULL DEFAULT 'private',
                parent_module TEXT,
                signature   TEXT,
                doc_comment TEXT
            );

            -- FTS5 full-text search trên name + doc_comment
            CREATE VIRTUAL TABLE IF NOT EXISTS symbols_fts USING fts5(
                name,
                doc_comment,
                content='symbols',
                content_rowid='rowid'
            );
            ",
        )?;
        Ok(())
    }

    /// Index toàn bộ symbols từ một danh sách (xóa dữ liệu cũ trước).
    pub fn index_all(&mut self, symbols: &[Symbol]) -> Result<usize> {
        let tx = self.conn.transaction()?;

        // Xóa dữ liệu cũ
        tx.execute("DELETE FROM symbols", [])?;

        let mut count = 0;
        for sym in symbols {
            insert_symbol_in_tx(&tx, sym)?;
            count += 1;
        }

        tx.commit()?;

        // Rebuild FTS index
        self.conn
            .execute("INSERT INTO symbols_fts(symbols_fts) VALUES('rebuild')", [])?;

        Ok(count)
    }

    /// Full-text search — tìm symbol theo tên / doc comment.
    pub fn search(&self, query: &str) -> Result<Vec<Symbol>> {
        let mut stmt = self.conn.prepare(
            "SELECT s.* FROM symbols s
             INNER JOIN symbols_fts fts ON s.rowid = fts.rowid
             WHERE symbols_fts MATCH ?1
             ORDER BY rank
             LIMIT 50",
        )?;

        let rows = stmt.query_map(params![query], row_to_symbol)?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    /// Tìm symbol theo tên chính xác.
    pub fn find_by_name(&self, name: &str) -> Result<Vec<Symbol>> {
        let mut stmt = self.conn.prepare(
            "SELECT * FROM symbols WHERE name = ?1",
        )?;
        let rows = stmt.query_map(params![name], row_to_symbol)?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    /// Tìm symbol theo kind và file.
    pub fn find_by_kind_and_file(
        &self,
        kind: Option<SymbolKind>,
        file: Option<&str>,
    ) -> Result<Vec<Symbol>> {
        let kind_str = kind.map(|k| k.as_str().to_string());

        if let (Some(k), Some(f)) = (&kind_str, file) {
            let mut stmt = self.conn.prepare(
                "SELECT * FROM symbols WHERE kind = ?1 AND file_path = ?2",
            )?;
            let rows = stmt.query_map(params![k, f], row_to_symbol)?;
            collect_rows(rows)
        } else if let Some(k) = kind_str {
            let mut stmt = self.conn.prepare(
                "SELECT * FROM symbols WHERE kind = ?1",
            )?;
            let rows = stmt.query_map(params![k], row_to_symbol)?;
            collect_rows(rows)
        } else if let Some(f) = file {
            let mut stmt = self.conn.prepare(
                "SELECT * FROM symbols WHERE file_path = ?1",
            )?;
            let rows = stmt.query_map(params![f], row_to_symbol)?;
            collect_rows(rows)
        } else {
            let mut stmt = self.conn.prepare("SELECT * FROM symbols")?;
            let rows = stmt.query_map([], row_to_symbol)?;
            collect_rows(rows)
        }
    }

    /// Đếm số lượng symbol theo kind.
    pub fn count_by_kind(&self) -> Result<Vec<(String, usize)>> {
        let mut stmt = self.conn.prepare(
            "SELECT kind, COUNT(*) as cnt FROM symbols GROUP BY kind ORDER BY cnt DESC",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, usize>(1)?))
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    /// Tổng số symbols trong index.
    pub fn total_symbols(&self) -> Result<usize> {
        let count: usize = self
            .conn
            .query_row("SELECT COUNT(*) FROM symbols", [], |row| row.get(0))?;
        Ok(count)
    }
}

// ── Helpers ──────────────────────────────────────────────────

/// Chèn một symbol vào transaction đang mở.
fn insert_symbol_in_tx(
    tx: &rusqlite::Transaction,
    sym: &Symbol,
) -> Result<()> {
    tx.execute(
        "INSERT INTO symbols (id, name, kind, file_path, line_start, line_end, column_start, visibility, parent_module, signature, doc_comment)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![
            sym.id.0,
            sym.name,
            sym.kind.as_str(),
            sym.file_path,
            sym.line_start as i64,
            sym.line_end as i64,
            sym.column_start as i64,
            visibility_to_str(sym.visibility),
            sym.parent_module,
            sym.signature,
            sym.doc_comment,
        ],
    )?;
    Ok(())
}

fn visibility_to_str(v: crate::Visibility) -> &'static str {
    match v {
        crate::Visibility::Public => "public",
        crate::Visibility::Crate => "crate",
        crate::Visibility::Private => "private",
    }
}

fn row_to_symbol(row: &rusqlite::Row) -> rusqlite::Result<Symbol> {
    let kind_str: String = row.get("kind")?;
    let kind = match kind_str.as_str() {
        "fn" => SymbolKind::Function,
        "struct" => SymbolKind::Struct,
        "enum" => SymbolKind::Enum,
        "trait" => SymbolKind::Trait,
        "impl" => SymbolKind::Impl,
        "mod" => SymbolKind::Module,
        "type" => SymbolKind::TypeAlias,
        "const" => SymbolKind::Const,
        "static" => SymbolKind::Static,
        "use" => SymbolKind::Use,
        "macro" => SymbolKind::Macro,
        _ => SymbolKind::Function, // fallback
    };

    let vis_str: String = row.get("visibility")?;
    let visibility = match vis_str.as_str() {
        "public" => crate::Visibility::Public,
        "crate" => crate::Visibility::Crate,
        _ => crate::Visibility::Private,
    };

    Ok(Symbol {
        id: crate::SymbolId(row.get("id")?),
        name: row.get("name")?,
        kind,
        file_path: row.get("file_path")?,
        line_start: row.get::<_, i64>("line_start")? as usize,
        line_end: row.get::<_, i64>("line_end")? as usize,
        column_start: row.get::<_, i64>("column_start")? as usize,
        visibility,
        parent_module: row.get("parent_module")?,
        signature: row.get("signature")?,
        doc_comment: row.get("doc_comment")?,
    })
}

fn collect_rows(
    rows: impl Iterator<Item = rusqlite::Result<Symbol>>,
) -> Result<Vec<Symbol>> {
    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }
    Ok(results)
}

// ── High-level API ───────────────────────────────────────────

/// Build symbol index từ một thư mục source.
/// Quét tất cả file .rs, parse, và lưu vào SQLite.
pub fn build_index(source_dir: &Path, db_path: &Path) -> Result<IndexResult> {
    let start = Instant::now();

    // Thu thập tất cả file .rs
    let mut files = Vec::new();
    collect_rust_files(source_dir, &mut files)?;

    let mut all_symbols = Vec::new();
    let mut errors = Vec::new();

    for file_path in &files {
        match std::fs::read_to_string(file_path) {
            Ok(source) => match crate::parser::parse_file(file_path, &source) {
                Ok(symbols) => all_symbols.extend(symbols),
                Err(e) => errors.push(format!("{}: {}", file_path.display(), e)),
            },
            Err(e) => errors.push(format!("{}: {}", file_path.display(), e)),
        }
    }

    let symbols_found = all_symbols.len();
    let files_processed = files.len();

    // Lưu vào database
    let mut index = SymbolIndex::open(db_path)?;
    index.index_all(&all_symbols)?;

    // Build symbol name map cho relation graph
    let symbol_names: std::collections::HashMap<String, crate::SymbolId> = all_symbols
        .iter()
        .map(|s| (s.name.clone(), s.id.clone()))
        .collect();

    // Build relation graph
    let (_edge_count, graph_errors) = crate::graph::build_graph(source_dir, db_path, &symbol_names)
        .unwrap_or((0, vec![]));
    errors.extend(graph_errors);

    Ok(IndexResult {
        files_processed,
        symbols_found,
        duration_ms: start.elapsed().as_millis() as u64,
        errors,
    })
}

/// Đệ quy thu thập tất cả file .rs trong thư mục.
pub(crate) fn collect_rust_files(dir: &Path, files: &mut Vec<std::path::PathBuf>) -> Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }

    for entry in std::fs::read_dir(dir)
        .with_context(|| format!("Không thể đọc thư mục: {}", dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();

        // Bỏ qua thư mục ẩn và target
        if let Some(name) = path.file_name() {
            let name = name.to_string_lossy();
            if name.starts_with('.') || name == "target" || name == "node_modules" {
                continue;
            }
        }

        if path.is_dir() {
            collect_rust_files(&path, files)?;
        } else if path.extension().map_or(false, |e| e == "rs") {
            files.push(path);
        }
    }
    Ok(())
}

// ── Tests ────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn index_and_search() {
        let mut index = SymbolIndex::open_in_memory().unwrap();

        let sym = Symbol {
            id: crate::SymbolId("test:1".into()),
            name: "authenticate".into(),
            kind: SymbolKind::Function,
            file_path: "src/auth.rs".into(),
            line_start: 42,
            line_end: 50,
            column_start: 0,
            visibility: crate::Visibility::Public,
            parent_module: None,
            signature: Some("pub fn authenticate(token: &str) -> bool".into()),
            doc_comment: Some("Xác thực người dùng bằng token.".into()),
        };

        index.index_all(&[sym]).unwrap();

        let results = index.search("xác thực").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "authenticate");
    }

    #[test]
    fn find_by_kind_and_file() {
        let mut index = SymbolIndex::open_in_memory().unwrap();

        let syms = vec![
            Symbol {
                id: crate::SymbolId("a:1".into()),
                name: "login".into(),
                kind: SymbolKind::Function,
                file_path: "src/auth.rs".into(),
                line_start: 1,
                line_end: 10,
                column_start: 0,
                visibility: crate::Visibility::Public,
                parent_module: None,
                signature: None,
                doc_comment: None,
            },
            Symbol {
                id: crate::SymbolId("a:2".into()),
                name: "User".into(),
                kind: SymbolKind::Struct,
                file_path: "src/models.rs".into(),
                line_start: 1,
                line_end: 5,
                column_start: 0,
                visibility: crate::Visibility::Public,
                parent_module: None,
                signature: None,
                doc_comment: None,
            },
        ];

        index.index_all(&syms).unwrap();

        let fns = index.find_by_kind_and_file(Some(SymbolKind::Function), None).unwrap();
        assert_eq!(fns.len(), 1);
        assert_eq!(fns[0].name, "login");

        let in_auth = index.find_by_kind_and_file(None, Some("src/auth.rs")).unwrap();
        assert_eq!(in_auth.len(), 1);
    }
}
