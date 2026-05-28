//! VietCode CLI — Giao diện dòng lệnh.

mod commands;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "vietcode")]
#[command(version = "0.1.0")]
#[command(about = "VietCode — Đội quân Agent code tiếng Việt", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Build symbol index từ codebase
    Index {
        #[arg(default_value = ".")]
        dir: String,
        #[arg(long, default_value = ".vietcode/index.db")]
        db: String,
    },

    /// Tìm kiếm symbols
    Query {
        query: String,
        #[arg(long)]
        kind: Option<String>,
        #[arg(long)]
        file: Option<String>,
        #[arg(long, default_value = ".vietcode/index.db")]
        db: String,
        #[arg(long, default_value = "table")]
        format: String,
    },

    /// Xem ai gọi function này (callers)
    Callers {
        /// Tên function cần tra
        name: String,
        #[arg(long, default_value = ".vietcode/index.db")]
        db: String,
    },

    /// Xem function này gọi những ai (callees)
    Callees {
        /// Tên function cần tra
        name: String,
        #[arg(long, default_value = ".vietcode/index.db")]
        db: String,
    },

    /// Phân tích impact: callers + callees
    Impact {
        /// Tên function cần phân tích
        name: String,
        #[arg(long, default_value = ".vietcode/index.db")]
        db: String,
    },

    /// Theo dõi thay đổi file
    Watch {
        #[arg(long, default_value = ".vietcode/index.db")]
        db: String,
    },

    /// Gửi task cho AI agent (Phase 3)
    Ask {
        task: String,
    },
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        match cli.command {
            Commands::Index { dir, db } => commands::index::run(&dir, &db)?,
            Commands::Query { query, kind, file, db, format } => {
                commands::query::run(&query, kind.as_deref(), file.as_deref(), &db, &format)?;
            }
            Commands::Callers { name, db } => commands::graph::run_callers(&name, &db)?,
            Commands::Callees { name, db } => commands::graph::run_callees(&name, &db)?,
            Commands::Impact { name, db } => commands::graph::run_impact(&name, &db)?,
            Commands::Watch { db } => commands::watch::run(&db)?,
            Commands::Ask { task } => commands::ask::run(&task)?,
        }
        Ok::<(), anyhow::Error>(())
    })?;

    Ok(())
}
