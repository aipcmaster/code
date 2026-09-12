//! 存储实体模型。

use serde::{Deserialize, Serialize};

/// 记录在库中的一条快照。
#[derive(Debug, Clone, PartialEq)]
pub struct SnapshotRecord {
    pub timestamp_unix_ms: u64,
    pub snapshot: aipcmaster_collect::SystemSnapshot,
}

/// 记录在库中的一份诊断报告。
#[derive(Debug, Clone, PartialEq)]
pub struct ReportRecord {
    pub session_id: String,
    pub health_score: u8,
    pub summary: String,
    pub issues: Vec<aipcmaster_diagnose::Issue>,
    pub generated_at_unix_ms: u64,
}

/// 审计日志类别。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditLogKind {
    /// 优化执行
    Optimize,
    /// 诊断
    Diagnose,
    /// 设置变更
    Setting,
    /// 其他
    Other,
}

impl AuditLogKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            AuditLogKind::Optimize => "optimize",
            AuditLogKind::Diagnose => "diagnose",
            AuditLogKind::Setting => "setting",
            AuditLogKind::Other => "other",
        }
    }
}

/// 一条审计日志（ERD §3.15 audit_logs 的本地镜像）。
#[derive(Debug, Clone, PartialEq)]
pub struct AuditLog {
    pub kind: AuditLogKind,
    pub action: String,
    pub target: Option<String>,
    pub detail: String,
    pub created_at_unix_ms: u64,
}
