//! 问题（issue）定义，对齐 ERD §3.10 `issues` 表。

use serde::{Deserialize, Serialize};

/// 问题类别（ERD §3.10 category）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
pub enum Category {
    /// 性能问题
    Performance,
    /// 磁盘问题
    Disk,
    /// 内存问题
    Memory,
    /// 安全问题
    Security,
}

impl std::fmt::Display for Category {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Category::Performance => write!(f, "performance"),
            Category::Disk => write!(f, "disk"),
            Category::Memory => write!(f, "memory"),
            Category::Security => write!(f, "security"),
        }
    }
}

/// 严重度。`Critical > High > Medium > Low`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

impl Severity {
    /// 严重度对应的健康分扣分。
    pub fn health_penalty(self) -> u8 {
        match self {
            Severity::Low => 5,
            Severity::Medium => 15,
            Severity::High => 30,
            Severity::Critical => 50,
        }
    }
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Low => write!(f, "low"),
            Severity::Medium => write!(f, "medium"),
            Severity::High => write!(f, "high"),
            Severity::Critical => write!(f, "critical"),
        }
    }
}

/// 检测到的一个问题（ERD §3.10 issues 表字段映射）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Issue {
    pub category: Category,
    pub severity: Severity,
    /// 问题标题，如 "CPU 使用率持续偏高"。
    pub title: String,
    /// 自然语言描述与建议。
    pub description: String,
    /// 证据（关键指标值），天然适配 ERD evidence_json。
    pub evidence: serde_json::Value,
}

impl Issue {
    pub fn new(
        category: Category,
        severity: Severity,
        title: impl Into<String>,
        description: impl Into<String>,
        evidence: serde_json::Value,
    ) -> Self {
        Self {
            category,
            severity,
            title: title.into(),
            description: description.into(),
            evidence,
        }
    }
}

// Serde 将枚举序列化为 snake_case 字符串，适配 DB/API。
impl Serialize for Category {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_string())
    }
}
impl<'de> Deserialize<'de> for Category {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        match s.as_str() {
            "performance" => Ok(Category::Performance),
            "disk" => Ok(Category::Disk),
            "memory" => Ok(Category::Memory),
            "security" => Ok(Category::Security),
            _ => Err(serde::de::Error::custom(format!("未知 category: {s}"))),
        }
    }
}

impl Serialize for Severity {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_string())
    }
}
impl<'de> Deserialize<'de> for Severity {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        match s.as_str() {
            "low" => Ok(Severity::Low),
            "medium" => Ok(Severity::Medium),
            "high" => Ok(Severity::High),
            "critical" => Ok(Severity::Critical),
            _ => Err(serde::de::Error::custom(format!("未知 severity: {s}"))),
        }
    }
}
