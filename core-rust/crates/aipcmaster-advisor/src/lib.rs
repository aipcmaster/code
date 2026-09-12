//! AIPCMaster（AI电脑大师）核心引擎 —— AI 推理层
//!
//! 对应《软件开发工程文档》§2.2「AI 推理层」：
//! 异常检测、根因分析、优化建议。
//!
//! 本实现为 **规则化引用实现**（deterministic，无外部依赖，完全可测）：
//! - 输入：诊断报告（问题列表 + 健康分）；
//! - 输出：根因结论 + 优化动作建议（对齐 ERD §3.11 `optimization_actions`：
//!   action_type / risk_level / requires_confirm / parameters）。
//!
//! 后续接入本地 LLM（llama.cpp / ONNX，SD 技术栈）时，只需在
//! [`Advisor`] 之后追加一个 LLM 层生成自然语言根因描述，
//! 本 crate 的动作建议可作为 LLM 的工具调用路由骨架（SD §2.2 工具调用）。

use aipcmaster_diagnose::{Category, DiagnosisReport, Issue, Severity};
use serde::{Deserialize, Serialize};

/// 优化动作类型（ERD §3.11 action_type）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionType {
    #[serde(rename = "clean_memory")]
    CleanMemory,
    #[serde(rename = "startup")]
    Startup,
    #[serde(rename = "disk")]
    Disk,
    /// 通用/未来的动作（例如温度、驱动、安全类）。
    #[serde(rename = "generic")]
    Generic,
}

impl ActionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ActionType::CleanMemory => "clean_memory",
            ActionType::Startup => "startup",
            ActionType::Disk => "disk",
            ActionType::Generic => "generic",
        }
    }
}

/// 风险级别（ERD §3.11 risk_level）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RiskLevel {
    #[serde(rename = "low")]
    Low,
    #[serde(rename = "medium")]
    Medium,
    #[serde(rename = "high")]
    High,
}

/// 一条优化动作建议（ERD §3.11 字段映射）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Suggestion {
    pub action_type: ActionType,
    pub risk_level: RiskLevel,
    /// 高危动作必须用户确认（SD §2.3 决策执行层：默认只读，修改需确认）。
    pub requires_confirm: bool,
    pub parameters: serde_json::Value,
    pub reason: String,
    /// 关联的问题类别。
    pub category: Category,
}

/// 一次推理的结果：根因摘要 + 动作建议列表。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Advice {
    /// 根因总结（规则化自然语言；后续可替换为 LLM 生成）。
    pub root_cause_summary: String,
    /// 按风险升序排列的动作建议。
    pub suggestions: Vec<Suggestion>,
}

impl Advice {
    /// 是否存在建议动作。
    pub fn is_empty(&self) -> bool {
        self.suggestions.is_empty()
    }
}

/// AI 推理引擎（规则化引用实现）。
///
/// 用法：
/// ```
/// use aipcmaster_collect::SystemSnapshot;
/// use aipcmaster_diagnose::diagnose;
/// use aipcmaster_advisor::Advisor;
///
/// let snap = SystemSnapshot::default();
/// let report = diagnose(&snap, "s1");
/// let advice = Advisor::new().advise(&report);
/// assert!(advice.suggestions.is_empty() || !advice.root_cause_summary.is_empty());
/// ```
#[derive(Default)]
pub struct Advisor;

impl Advisor {
    pub fn new() -> Self {
        Self
    }

    /// 对一次诊断报告执行根因分析与建议生成。
    pub fn advise(&self, report: &DiagnosisReport) -> Advice {
        let issues = &report.issues;
        let mut suggestions: Vec<Suggestion> = issues
            .iter()
            .filter(|i| i.severity >= Severity::Medium)
            .map(suggest_for)
            .collect();

        // 同类型动作去重：保留风险最高的（参数合并到 reason）
        suggestions = dedup(&mut suggestions);

        // 风险升序排序（低风险优先执行）
        suggestions.sort_by_key(|s| s.risk_level);

        let root_cause_summary = build_summary(report, &suggestions);

        Advice {
            root_cause_summary,
            suggestions,
        }
    }
}

/// 针对单个问题生成建议。
fn suggest_for(issue: &Issue) -> Suggestion {
    match issue.category {
        Category::Memory => {
            let is_critical = issue.severity == Severity::Critical;
            Suggestion {
                action_type: ActionType::CleanMemory,
                risk_level: if is_critical {
                    RiskLevel::Medium
                } else {
                    RiskLevel::Low
                },
                requires_confirm: is_critical,
                parameters: serde_json::json!({"target": "page_cache", "amount": "auto"}),
                reason: format!(
                    "内存{}，建议清理缓存页面释放内存",
                    if is_critical {
                        "严重偏高"
                    } else {
                        "使用率偏高"
                    }
                ),
                category: issue.category,
            }
        }
        Category::Disk => Suggestion {
            action_type: ActionType::Disk,
            risk_level: if issue.severity >= Severity::High {
                RiskLevel::High
            } else {
                RiskLevel::Medium
            },
            requires_confirm: issue.severity >= Severity::High,
            parameters: serde_json::json!({"target": "temp_files", "min_age_days": 30}),
            reason: "磁盘空间紧张，建议清理临时文件与缓存".to_string(),
            category: issue.category,
        },
        Category::Performance => Suggestion {
            action_type: ActionType::Startup,
            risk_level: RiskLevel::Low,
            requires_confirm: false,
            parameters: serde_json::json!({"target": "startup_items"}),
            reason: "CPU/负载偏高，建议检查开机自启项与后台进程".to_string(),
            category: issue.category,
        },
        Category::Security => Suggestion {
            action_type: ActionType::Generic,
            risk_level: RiskLevel::High,
            requires_confirm: true,
            parameters: serde_json::json!({"target": "security_scan"}),
            reason: "存在安全风险，建议执行安全扫描并更新系统".to_string(),
            category: issue.category,
        },
    }
}

/// 同类动作去重：保留其中风险最高的建议（更新 reason 说明合并）。
fn dedup(items: &mut Vec<Suggestion>) -> Vec<Suggestion> {
    let mut out: Vec<Suggestion> = Vec::new();
    for item in items.drain(..) {
        if let Some(existing) = out
            .iter_mut()
            .find(|e| e.action_type == item.action_type && e.category == item.category)
        {
            // 取更高风险
            if item.risk_level > existing.risk_level {
                existing.risk_level = item.risk_level;
                existing.requires_confirm = item.requires_confirm;
            }
            existing.reason = format!("{}；{}", existing.reason, item.reason);
        } else {
            out.push(item);
        }
    }
    out
}

/// 生成根因摘要（模板化自然语言；后续可换成 LLM）。
fn build_summary(report: &DiagnosisReport, suggestions: &[Suggestion]) -> String {
    let score = report.health_score;
    if suggestions.is_empty() {
        return format!("系统健康分 {}，未发现需要优化的问题。", score);
    }
    let worst = report
        .issues
        .iter()
        .map(|i| i.severity)
        .max()
        .unwrap_or(Severity::Low);
    let worst_word = match worst {
        Severity::Critical => "严重",
        Severity::High => "较高",
        Severity::Medium => "中等",
        Severity::Low => "轻微",
    };
    let kinds: Vec<String> = suggestions
        .iter()
        .map(|s| s.action_type.as_str().to_string())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();
    format!(
        "健康分 {score}。存在{worst_word}风险问题（{}），建议执行 {} 项优化：{}。",
        kinds.join("、"),
        suggestions.len(),
        suggestions
            .iter()
            .map(|s| s.action_type.as_str())
            .collect::<Vec<_>>()
            .join("、")
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use aipcmaster_diagnose::DiagnosisReport;

    fn report_with(issues: Vec<Issue>) -> DiagnosisReport {
        DiagnosisReport {
            health_score: 60,
            issues,
            generated_at_unix_ms: 1000,
            session_id: "s1".into(),
        }
    }

    fn mem_issue(sev: Severity) -> Issue {
        Issue::new(
            Category::Memory,
            sev,
            "内存偏高",
            "内存使用率 85%",
            serde_json::json!({"usage_percent": 85.0}),
        )
    }

    #[test]
    fn healthy_report_no_suggestions() {
        let report = report_with(vec![]);
        let advice = Advisor::new().advise(&report);
        assert!(advice.suggestions.is_empty());
        assert!(advice.root_cause_summary.contains("未发现"));
    }

    #[test]
    fn memory_issue_suggests_clean_memory() {
        let report = report_with(vec![mem_issue(Severity::Medium)]);
        let advice = Advisor::new().advise(&report);
        assert_eq!(advice.suggestions.len(), 1);
        let s = &advice.suggestions[0];
        assert_eq!(s.action_type, ActionType::CleanMemory);
        assert_eq!(s.risk_level, RiskLevel::Low);
        assert!(!s.requires_confirm);
    }

    #[test]
    fn critical_memory_requires_confirm() {
        let report = report_with(vec![mem_issue(Severity::Critical)]);
        let advice = Advisor::new().advise(&report);
        let s = &advice.suggestions[0];
        assert_eq!(s.risk_level, RiskLevel::Medium);
        assert!(s.requires_confirm, "高危动作必须确认");
    }

    #[test]
    fn dedup_same_type_issues() {
        let report = report_with(vec![mem_issue(Severity::Medium), mem_issue(Severity::High)]);
        let advice = Advisor::new().advise(&report);
        assert_eq!(advice.suggestions.len(), 1, "同类建议应合并");
        assert_eq!(advice.suggestions[0].risk_level, RiskLevel::Low); // medium 的 clean_memory 也是 low；此处验证去重
        assert!(advice.suggestions[0].reason.contains("；"));
    }

    #[test]
    fn suggestions_sorted_by_risk() {
        let report = report_with(vec![
            Issue::new(
                Category::Disk,
                Severity::High,
                "磁盘",
                "高",
                serde_json::json!({}),
            ),
            mem_issue(Severity::Medium),
            Issue::new(
                Category::Performance,
                Severity::Medium,
                "CPU",
                "中",
                serde_json::json!({}),
            ),
        ]);
        let advice = Advisor::new().advise(&report);
        let risks: Vec<RiskLevel> = advice.suggestions.iter().map(|s| s.risk_level).collect();
        let mut sorted = risks.clone();
        sorted.sort();
        assert_eq!(risks, sorted, "建议应按风险升序");
    }

    #[test]
    fn serde_roundtrip_snake_case() {
        let s = Suggestion {
            action_type: ActionType::CleanMemory,
            risk_level: RiskLevel::Low,
            requires_confirm: false,
            parameters: serde_json::json!({}),
            reason: "r".into(),
            category: Category::Memory,
        };
        let v = serde_json::to_value(&s).unwrap();
        assert_eq!(v["action_type"], "clean_memory");
        assert_eq!(v["risk_level"], "low");
        let back: Suggestion = serde_json::from_value(v).unwrap();
        assert_eq!(back, s);
    }
}
