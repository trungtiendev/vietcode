//! File Watcher — Incremental update cho Cortex.
//!
//! Theo dõi thay đổi file trong codebase và cập nhật index.
//! Phase 1: polling đơn giản. Phase 3: notify crate.

use anyhow::Result;

/// Placeholder cho file watcher.
pub struct FileWatcher;

impl FileWatcher {
    pub fn new() -> Self {
        Self
    }

    /// Chạy watcher — hiện tại là no-op.
    pub fn watch(&self) -> Result<()> {
        tracing::info!("File watcher started (no-op in Phase 1)");
        Ok(())
    }
}
