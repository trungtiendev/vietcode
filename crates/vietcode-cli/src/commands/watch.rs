//! `vietcode watch` — File watcher.

use anyhow::Result;

pub fn run(_db: &str) -> Result<()> {
    println!("vietcode watch — đang theo dõi thay đổi file...");
    println!("(Phase 1: polling đơn giản, Phase 3: notify real-time)");
    println!("Nhấn Ctrl+C để dừng.");

    let mut iteration = 0;
    loop {
        std::thread::sleep(std::time::Duration::from_secs(10));
        iteration += 1;
        println!("  [check #{}] — chưa có thay đổi", iteration);
    }
}
