//! `vietcode index` — Build symbol index.

use anyhow::Result;
use std::path::Path;
use vietcode_cortex::index;

pub fn run(dir: &str, db: &str) -> Result<()> {
    let source_dir = Path::new(dir);
    if !source_dir.exists() {
        anyhow::bail!("Thư mục không tồn tại: {}", dir);
    }

    // Tạo thư mục .vietcode nếu chưa có
    if let Some(parent) = Path::new(db).parent() {
        std::fs::create_dir_all(parent)?;
    }

    let db_path = Path::new(db);
    println!("Đang index codebase từ: {}", dir);
    println!("Database: {}\n", db);

    let result = index::build_index(source_dir, db_path)?;

    println!("Hoàn thành:");
    println!("  Files processed: {}", result.files_processed);
    println!("  Symbols found:  {}", result.symbols_found);
    println!("  Thời gian:      {}ms", result.duration_ms);

    if !result.errors.is_empty() {
        println!("\nLỗi ({} file):", result.errors.len());
        for err in &result.errors {
            println!("  - {}", err);
        }
    }

    // Hiển thị thống kê theo kind
    let idx = vietcode_cortex::index::SymbolIndex::open(db_path)?;
    let counts = idx.count_by_kind()?;
    println!("\nThống kê theo loại:");
    for (kind, count) in counts {
        println!("  {:>8}: {}", kind, count);
    }

    // Hiển thị thống kê graph
    if let Ok(graph) = vietcode_cortex::graph::RelationGraph::open(db_path)
        && let Ok(total) = graph.total_edges()
            && total > 0 {
                println!("\nRelation graph:");
                println!("  Total edges: {}", total);
                if let Ok(by_type) = graph.count_by_type() {
                    for (etype, count) in by_type {
                        println!("    {}: {}", etype, count);
                    }
                }
            }

    Ok(())
}
