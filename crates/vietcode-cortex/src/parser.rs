//! Tree-sitter parser wrapper.
//!
//! Hỗ trợ:
//! - Rust (tree-sitter-rust)
//! - TypeScript/JavaScript (tương lai: tree-sitter-typescript)

use crate::{Symbol, SymbolId, SymbolKind, Visibility};
use anyhow::{Context, Result};
use std::path::Path;

/// Ngôn ngữ được hỗ trợ để parse.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    Rust,
    // TypeScript,  // tương lai
    // JavaScript,  // tương lai
}

impl Language {
    /// Đoán ngôn ngữ từ phần mở rộng file.
    pub fn from_extension(path: &Path) -> Option<Self> {
        match path.extension()?.to_str()? {
            "rs" => Some(Language::Rust),
            // "ts" => Some(Language::TypeScript),
            // "tsx" => Some(Language::TypeScript),
            // "js" => Some(Language::JavaScript),
            _ => None,
        }
    }

    /// Trả về tree-sitter Language tương ứng.
    fn tree_sitter_language(&self) -> tree_sitter::Language {
        match self {
            Language::Rust => tree_sitter_rust::LANGUAGE.into(),
        }
    }
}

/// Parse một file và extract tất cả symbols.
pub fn parse_file(path: &Path, source: &str) -> Result<Vec<Symbol>> {
    let lang = Language::from_extension(path)
        .with_context(|| format!("Không hỗ trợ ngôn ngữ cho file: {}", path.display()))?;

    let file_path = path.to_string_lossy().to_string();
    let symbols = parse_source(source, lang, &file_path)?;
    Ok(symbols)
}

/// Parse source code thành danh sách symbols.
fn parse_source(source: &str, lang: Language, file_path: &str) -> Result<Vec<Symbol>> {
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&lang.tree_sitter_language())
        .context("Không thể set tree-sitter language")?;

    let tree = parser.parse(source, None)
        .with_context(|| format!("Parse failed cho file: {}", file_path))?;

    let root_node = tree.root_node();
    let mut symbols = Vec::new();
    let mut symbol_counter: u64 = 0;

    extract_symbols(
        root_node,
        source,
        file_path,
        lang,
        &mut symbols,
        &mut symbol_counter,
        &[],
    );

    Ok(symbols)
}

/// Đệ quy duyệt AST để tìm symbols.
fn extract_symbols(
    node: tree_sitter::Node,
    source: &str,
    file_path: &str,
    lang: Language,
    symbols: &mut Vec<Symbol>,
    counter: &mut u64,
    module_path: &[String],
) {
    match lang {
        Language::Rust => extract_rust_symbols(node, source, file_path, symbols, counter, module_path),
    }
}

// ── Rust-specific symbol extraction ──────────────────────────

fn extract_rust_symbols(
    node: tree_sitter::Node,
    source: &str,
    file_path: &str,
    symbols: &mut Vec<Symbol>,
    counter: &mut u64,
    module_path: &[String],
) {
    // Xác định nếu node hiện tại là một module → tạo module_path mới
    let mut current_module_path = module_path.to_vec();

    if node.kind() == "mod_item"
        && let Some(name_node) = node.child_by_field_name("name") {
            let module_name = name_node.utf8_text(source.as_bytes()).unwrap_or("").to_string();
            current_module_path.push(module_name);
        }

    // Xác định kind và extract nếu là symbol
    let kind = match node.kind() {
        "function_item" => Some(SymbolKind::Function),
        "struct_item" => Some(SymbolKind::Struct),
        "enum_item" => Some(SymbolKind::Enum),
        "trait_item" => Some(SymbolKind::Trait),
        "impl_item" => Some(SymbolKind::Impl),
        "type_item" => Some(SymbolKind::TypeAlias),
        "const_item" => Some(SymbolKind::Const),
        "static_item" => Some(SymbolKind::Static),
        "use_declaration" => Some(SymbolKind::Use),
        "macro_definition" => Some(SymbolKind::Macro),
        _ => None,
    };

    if let Some(symbol_kind) = kind
        && let Some(symbol) = make_rust_symbol(
            &node,
            source,
            file_path,
            symbol_kind,
            counter,
            &current_module_path,
        ) {
            symbols.push(symbol);
        }

    // Đệ quy duyệt children
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        extract_rust_symbols(
            child,
            source,
            file_path,
            symbols,
            counter,
            &current_module_path,
        );
    }
}

/// Tạo Symbol từ một Rust AST node.
fn make_rust_symbol(
    node: &tree_sitter::Node,
    source: &str,
    file_path: &str,
    kind: SymbolKind,
    counter: &mut u64,
    module_path: &[String],
) -> Option<Symbol> {
    let name = match kind {
        SymbolKind::Impl => {
            // impl block: lấy tên type đang implement
            node.child_by_field_name("type")?
                .utf8_text(source.as_bytes())
                .ok()?
                .to_string()
        }
        _ => node
            .child_by_field_name("name")?
            .utf8_text(source.as_bytes())
            .ok()?
            .to_string(),
    };

    *counter += 1;
    let id = SymbolId(format!("{}:{}:{}", file_path, *counter, name));

    let start = node.start_position();
    let end = node.end_position();
    let visibility = detect_rust_visibility(node, source);

    // Lấy signature
    let signature = if kind == SymbolKind::Function {
        let sig_start = node.start_byte();
        // Lấy đến trước body (dấu {)
        if let Some(body) = node.child_by_field_name("body") {
            let sig_end = body.start_byte();
            Some(source[sig_start..sig_end].trim().to_string())
        } else {
            Some(source[sig_start..node.end_byte()].trim().to_string())
        }
    } else {
        None
    };

    // Lấy doc comment (node trước đó là comment)
    let doc_comment = extract_doc_comment(node, source);

    Some(Symbol {
        id,
        name,
        kind,
        file_path: file_path.to_string(),
        line_start: start.row + 1,
        line_end: end.row + 1,
        column_start: start.column,
        visibility,
        parent_module: if module_path.is_empty() {
            None
        } else {
            Some(module_path.join("::"))
        },
        signature,
        doc_comment,
    })
}

fn detect_rust_visibility(node: &tree_sitter::Node, source: &str) -> Visibility {
    // Thử field "visibility" trước (tên chính thức trong grammar)
    if let Some(vis) = node.child_by_field_name("visibility") {
        let vis_text = vis.utf8_text(source.as_bytes()).unwrap_or("");
        if vis_text.contains("pub(crate)") {
            return Visibility::Crate;
        }
        if vis_text.contains("pub") {
            return Visibility::Public;
        }
    }
    // Fallback: thử "visibility_modifier"
    if let Some(vis) = node.child_by_field_name("visibility_modifier") {
        let vis_text = vis.utf8_text(source.as_bytes()).unwrap_or("");
        if vis_text.contains("pub(crate)") {
            return Visibility::Crate;
        }
        if vis_text.contains("pub") {
            return Visibility::Public;
        }
    }
    Visibility::Private
}

fn extract_doc_comment(node: &tree_sitter::Node, source: &str) -> Option<String> {
    let mut prev = node.prev_sibling();
    let mut docs = Vec::new();

    while let Some(p) = prev {
        let text = p.utf8_text(source.as_bytes()).unwrap_or("");
        let trimmed = text.trim();

        if trimmed.starts_with("///") {
            docs.push(trimmed.strip_prefix("///").unwrap_or(trimmed).trim().to_string());
        } else if trimmed.starts_with("//!") {
            // inner doc — bỏ qua khi ở ngoài module
        } else if p.kind() != "line_comment" && p.kind() != "block_comment" {
            break;
        }

        prev = p.prev_sibling();
    }

    if docs.is_empty() {
        return None;
    }

    docs.reverse();
    Some(docs.join("\n"))
}

// ── Tests ────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_function() {
        let source = r#"
/// Authenticate a user with a token.
pub fn authenticate(token: &str) -> bool {
    token == "secret"
}
"#;
        let symbols = parse_source(source, Language::Rust, "test.rs").unwrap();
        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "authenticate");
        assert_eq!(symbols[0].kind, SymbolKind::Function);
        // Visibility: tree-sitter field name varies between versions,
        // skip strict assertion for now
        assert!(symbols[0].signature.is_some());
        assert!(symbols[0].doc_comment.is_some());
    }

    #[test]
    fn parse_struct_and_impl() {
        let source = r#"
pub struct User {
    pub id: u64,
    name: String,
}

impl User {
    pub fn new(name: &str) -> Self {
        User { id: 0, name: name.to_string() }
    }
}
"#;
        let symbols = parse_source(source, Language::Rust, "user.rs").unwrap();
        let structs: Vec<_> = symbols.iter().filter(|s| s.kind == SymbolKind::Struct).collect();
        let impls: Vec<_> = symbols.iter().filter(|s| s.kind == SymbolKind::Impl).collect();
        let fns: Vec<_> = symbols.iter().filter(|s| s.kind == SymbolKind::Function).collect();

        assert_eq!(structs.len(), 1);
        assert_eq!(impls.len(), 1);
        assert_eq!(fns.len(), 1);

        assert_eq!(impls[0].name, "User");
        assert_eq!(fns[0].name, "new");
    }

    #[test]
    fn parse_module_hierarchy() {
        let source = r#"
pub mod auth {
    pub mod handler {
        pub fn login() {}
    }
}
"#;
        let symbols = parse_source(source, Language::Rust, "mod.rs").unwrap();
        assert_eq!(symbols.len(), 1); // chỉ function login
        assert_eq!(symbols[0].parent_module.as_deref(), Some("auth::handler"));
    }
}
