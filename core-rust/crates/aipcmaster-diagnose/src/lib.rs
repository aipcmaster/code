//! AIPCMaster（AI电脑大师）核心引擎 —— 诊断引擎
//!
//! 对应《AIPCMaster（AI电脑大师）软件开发工程文档》§3.1「诊断引擎模块」
//! 与《PRD》US-02：一键诊断/自然语言提问 → 健康分 + 问题列表 + 证据 + 建议。
//!
//! 设计原则：
//! - **纯计算**：输入 `SystemSnapshot`，输出 `DiagnosisReport`，无 I/O，完全可测；
//! - **规则可扩展**：诊断规则为 `Vec<Box<dyn DiagnosticRule>>`，可插拔叠加
//!   （后续 AI 推理层可作为最高级的规则接入）；
//! - **对齐 ERD**：issue 的 category/severity 字段与 `issues` 表一一对应。

mod health;
mod issue;
mod rule;
mod snapshot_rules;

pub use health::compute_health_score;
pub use issue::{Category, Issue, Severity};
pub use rule::{DiagnosticRule, RuleVerdict};
pub use snapshot_rules::standard_rules;

use aipcmaster_collect::SystemSnapshot;
use serde::{Deserialize, Serialize};

/// 一次诊断的结果。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiagnosisReport {
    /// 健康分 0~100（100 为最佳）。
    pub health_score: u8,
    /// 检测到的问题列表（按严重度降序）。
    pub issues: Vec<Issue>,
    /// 诊断完成时间（Unix 毫秒）。
    pub generated_at_unix_ms: u64,
    /// 用于定位的会话标识（由上层生成，诊断引擎不计生成）。
    pub session_id: String,
}

impl DiagnosisReport {
    pub fn issues_by_severity(&self) -> impl Iterator<Item = &Issue> {
        self.issues.iter()
    }
}

/// 诊断引擎：对一次系统快照执行全部标准规则。
///
/// 用法：`DiagnosticEngine::new()` 后对每次 `SystemSnapshot` 调用 [`Engine::diagnose`]。
#[derive(Default)]
pub struct DiagnosticEngine {
    rules: Vec<Box<dyn DiagnosticRule>>,
}

impl DiagnosticEngine {
    /// 使用全部标准规则构建引擎。
    pub fn new() -> Self {
        Self {
            rules: standard_rules(),
        }
    }

    /// 自定义规则集。
    pub fn with_rules(rules: Vec<Box<dyn DiagnosticRule>>) -> Self {
        Self { rules }
    }

    /// 对快照执行诊断。
    pub fn diagnose(&self, snap: &SystemSnapshot, session_id: &str) -> DiagnosisReport {
        let mut issues: Vec<Issue> = self.rules.iter().filter_map(|r| r.check(snap)).collect();
        issues.sort_by_key(|i| std::cmp::Reverse(i.severity));

        DiagnosisReport {
            health_score: compute_health_score(&issues),
            issues,
            generated_at_unix_ms: snap.timestamp_unix_ms,
            session_id: session_id.to_string(),
        }
    }
}

/// 快速入口：快照 → 报告。方便二元面包屑与示例使用。
pub fn diagnose(snap: &SystemSnapshot, session_id: &str) -> DiagnosisReport {
    DiagnosticEngine::new().diagnose(snap, session_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use aipcmaster_collect::{
        CpuMetrics, DiskMetrics, MemoryMetrics, NetworkMetrics, ProcessMetrics, SystemSnapshot,
    };

    /// 构造一个完全空闲的系统快照。
    fn healthy_snapshot() -> SystemSnapshot {
        SystemSnapshot {
            timestamp_unix_ms: 1000,
            cpu: Some(CpuMetrics {
                usage_percent: 5.0,
                per_core_usage_percent: vec![5.0; 8],
                load_avg_1: 0.5,
                load_avg_5: 0.5,
                load_avg_15: 0.5,
                core_count: 8,
                temperature_c: Some(45.0),
            }),
            memory: Some(MemoryMetrics {
                total_bytes: 16_000_000_000,
                used_bytes: 4_000_000_000,
                available_bytes: 12_000_000_000,
                usage_percent: 25.0,
                swap_total_bytes: 8_000_000_000,
                swap_used_bytes: 0,
            }),
            disks: vec![DiskMetrics {
                device: "/dev/sda1".into(),
                mount_point: "/".into(),
                fs_type: "ext4".into(),
                total_bytes: 500_000_000_000,
                used_bytes: 100_000_000_000,
                available_bytes: 400_000_000_000,
                usage_percent: 20.0,
            }],
            disk_io: vec![],
            networks: vec![NetworkMetrics {
                interface: "enp0s3".into(),
                rx_bytes: 0,
                tx_bytes: 0,
                rx_errors: 0,
                tx_errors: 0,
            }],
            processes: vec![ProcessMetrics {
                pid: 1,
                name: "systemd".into(),
                state: "S".into(),
                memory_bytes: 0,
            }],
        }
    }

    #[test]
    fn healthy_system_no_issues() {
        let snap = healthy_snapshot();
        let report = DiagnosticEngine::new().diagnose(&snap, "s1");
        assert!(report.issues.is_empty(), "非预期问题: {:?}", report.issues);
        assert_eq!(report.health_score, 100);
    }

    #[test]
    fn high_cpu_detected() {
        let mut snap = healthy_snapshot();
        snap.cpu.as_mut().unwrap().usage_percent = 95.0;
        snap.cpu.as_mut().unwrap().per_core_usage_percent = vec![95.0; 8];
        let report = diagnose(&snap, "s2");

        assert!(report
            .issues
            .iter()
            .any(|i| i.category == Category::Performance));
        assert!(report.health_score < 100);
    }

    #[test]
    fn severity_ordering() {
        let mut snap = healthy_snapshot();
        snap.cpu.as_mut().unwrap().usage_percent = 95.0;
        snap.memory.as_mut().unwrap().usage_percent = 96.0;
        snap.disks[0].usage_percent = 92.0;
        let report = diagnose(&snap, "s3");

        // 按严重度降序
        let s: Vec<Severity> = report.issues.iter().map(|i| i.severity).collect();
        let mut sorted = s.clone();
        sorted.sort_by_key(|x| std::cmp::Reverse(*x));
        assert_eq!(s, sorted);
    }
}
