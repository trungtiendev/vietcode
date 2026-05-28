//! `vietcode callers|callees|impact` — Query relation graph.

use anyhow::Result;
use std::path::Path;
use vietcode_cortex::graph::RelationGraph;

pub fn run_callers(name: &str, db: &str) -> Result<()> {
    let db_path = Path::new(db);
    ensure_db(db_path)?;

    let graph = RelationGraph::open(db_path)?;
    let callers = graph.callers(name)?;

    if callers.is_empty() {
        println!("Không có ai gọi '{}'", name);
    } else {
        println!("{} function gọi '{}':\n", callers.len(), name);
        for c in &callers {
            println!("  {}  ({}:{})", c.name, c.file_path, c.line);
        }
    }

    Ok(())
}

pub fn run_callees(name: &str, db: &str) -> Result<()> {
    let db_path = Path::new(db);
    ensure_db(db_path)?;

    let graph = RelationGraph::open(db_path)?;
    let callees = graph.callees(name)?;

    if callees.is_empty() {
        println!("'{}' không gọi function nào", name);
    } else {
        println!("'{}' gọi {} function:\n", name, callees.len());
        for c in &callees {
            println!("  {}  ({}:{})", c.name, c.file_path, c.line);
        }
    }

    Ok(())
}

pub fn run_impact(name: &str, db: &str) -> Result<()> {
    let db_path = Path::new(db);
    ensure_db(db_path)?;

    let graph = RelationGraph::open(db_path)?;
    let impact = graph.impact(name)?;

    println!("Impact analysis cho '{}':\n", name);

    if impact.callers.is_empty() {
        println!("  Callers: (không có)");
    } else {
        println!("  Callers ({} function gọi nó):", impact.callers.len());
        for c in &impact.callers {
            println!("    {}  ({}:{})", c.name, c.file_path, c.line);
        }
    }

    println!();

    if impact.callees.is_empty() {
        println!("  Callees: (không gọi ai)");
    } else {
        println!("  Callees (nó gọi {} function):", impact.callees.len());
        for c in &impact.callees {
            println!("    {}  ({}:{})", c.name, c.file_path, c.line);
        }
    }

    Ok(())
}

fn ensure_db(db_path: &Path) -> Result<()> {
    if !db_path.exists() {
        anyhow::bail!(
            "Database chưa tồn tại. Chạy `vietcode index` trước.\n  Path: {}",
            db_path.display()
        );
    }
    Ok(())
}
