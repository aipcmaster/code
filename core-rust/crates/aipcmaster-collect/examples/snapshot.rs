//! 示例：打印一次系统快照（JSON）。
//!
//! 用法：
//!   cargo run -p aipcmaster-collect --example snapshot
//!   cargo run -p aipcmaster-collect --example snapshot -- --full

use aipcmaster_collect::system_collector;

fn main() {
    let full = std::env::args().any(|a| a == "--full");
    let collector = system_collector().expect("当前平台不受支持");

    let snap = if full {
        collector.collect_full()
    } else {
        collector.collect()
    }
    .expect("采集失败");

    println!(
        "{}",
        serde_json::to_string_pretty(&snap).expect("序列化失败")
    );
}
