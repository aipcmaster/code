//! 示例：采集一次完整快照并现场诊断，输出健康分 + 问题列表（JSON）。
//!
//! 用法：
//!   cargo run -p aipcmaster-diagnose --example diagnose

use aipcmaster_collect::system_collector;
use aipcmaster_diagnose::DiagnosticEngine;
use serde_json::json;

fn main() {
    let collector = system_collector().expect("当前平台不受支持");
    let snap = collector.collect_full().expect("采集失败");
    let report = DiagnosticEngine::new().diagnose(&snap, "example");

    // 可读格式摘要
    println!("== 健康分: {}/100 ==", report.health_score);
    if report.issues.is_empty() {
        println!("未发现问题，系统状态良好。");
    }
    for (i, issue) in report.issues.iter().enumerate() {
        println!(
            "[{i}] [{}/{}] {}（{}）",
            issue.category, issue.severity, issue.title, issue.description
        );
    }

    // 完整 JSON（对齐诊断报告云上传格式）
    let payload = json!({
        "health_score": report.health_score,
        "issues": report.issues,
        "device_id": "example-device",
        "generated_at_unix_ms": report.generated_at_unix_ms,
    });
    println!(
        "\n{}",
        serde_json::to_string_pretty(&payload).expect("序列化失败")
    );
}
