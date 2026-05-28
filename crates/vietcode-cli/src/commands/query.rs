//! `vietcode query` — Tìm kiếm symbols.

use anyhow::Result;
use std::path::Path;
use vietcode_cortex::index::SymbolIndex;
use vietcode_cortex::SymbolKind;

pub fn run(
    query: &str,
    kind: Option<&str>,
    file: Option<&str>,
    db: &str,
    format: &str,
) -> Result<()> {
    let db_path = Path::new(db);
    if !db_path.exists() {
        anyhow::bail!(
            "Database chưa tồn tại. Chạy `vietcode index` trước.\n  Path: {}",
            db
        );
    }

    let idx = SymbolIndex::open(db_path)?;
    let kind_enum = kind.map(|k| match k {
        "fn" => SymbolKind::Function,
        "struct" => SymbolKind::Struct,
        "enum" => SymbolKind::Enum,
        "trait" => SymbolKind::Trait,
        "impl" => SymbolKind::Impl,
        "mod" => SymbolKind::Module,
        "type" => SymbolKind::TypeAlias,
        "const" => SymbolKind::Const,
        "static" => SymbolKind::Static,
        _ => SymbolKind::Function, // fallback
    });

    let symbols = if kind_enum.is_some() || file.is_some() {
        idx.find_by_kind_and_file(kind_enum, file)?
    } else {
        idx.search(query)?
    };

    if symbols.is_empty() {
        println!("Không tìm thấy kết quả cho: {}", query);
        return Ok(());
    }

    match format {
        "json" => {
            let json = serde_json::to_string_pretty(&symbols)?;
            println!("{}", json);
        }
        _ => {
            println!("\nTìm thấy {} kết quả:\n", symbols.len());
            for sym in &symbols {
                let vis = match sym.visibility {
                    vietcode_cortex::Visibility::Public => "pub",
                    vietcode_cortex::Visibility::Crate => "pub(crate)",
                    vietcode_cortex::Visibility::Private => "",
                };

                let location = format!("{}:{}", sym.file_path, sym.line_start);
                let module = sym.parent_module.as_deref().unwrap_or("");

                if vis.is_empty() {
                    println!("  {:>6}  {:<40}  {}", sym.kind.as_str(), sym.name, location);
                } else {
                    println!("  {:>3} {:>6}  {:<40}  {}", vis, sym.kind.as_str(), sym.name, location);
                }

                if !module.is_empty() {
                    println!("         module: {}", module);
                }
                if let Some(ref sig) = sym.signature {
                    if sig.len() < 100 {
                        println!("         sig:    {}", sig);
                    } else {
                        println!("         sig:    {}...", &sig[..97]);
                    }
                }
                if let Some(ref doc) = sym.doc_comment {
                    let first_line = doc.lines().next().unwrap_or("");
                    println!("         doc:    {}", first_line);
                }
                println!();
            }
        }
    }

    Ok(())
}
