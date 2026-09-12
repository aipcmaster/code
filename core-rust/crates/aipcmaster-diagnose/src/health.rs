//! 健康分计算。

use crate::issue::{Issue, Severity};

/// 由问题列表计算健康分（0~100）。
///
/// 算法（简单、可解释、可测试）：
/// - 无问题 → 100；
/// - 取严重度最高的最多 **3** 个问题扣分（防止几十个 Low 把分打到 0）；
/// - 扣分：Critical 40 / High 25 / Medium 15 / Low 5；
/// - 下限 20：症状层面的扣分不会打到 0，为后续 AI 根因层留出低分区间。
pub fn compute_health_score(issues: &[Issue]) -> u8 {
    if issues.is_empty() {
        return 100;
    }

    let mut sorted: Vec<&Issue> = issues.iter().collect();
    sorted.sort_by_key(|i| std::cmp::Reverse(i.severity));

    let penalty: u16 = sorted.iter().take(3).map(|i| penalty(i.severity)).sum();
    100u16.saturating_sub(penalty).max(20) as u8
}

fn penalty(s: Severity) -> u16 {
    match s {
        Severity::Critical => 40,
        Severity::High => 25,
        Severity::Medium => 15,
        Severity::Low => 5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::issue::{Category, Issue};
    use serde_json::json;

    fn issue(s: Severity) -> Issue {
        Issue::new(Category::Performance, s, "t", "d", json!({}))
    }

    #[test]
    fn empty_is_100() {
        assert_eq!(compute_health_score(&[]), 100);
    }

    #[test]
    fn single_issue_deducts() {
        assert_eq!(compute_health_score(&[issue(Severity::Critical)]), 60);
        assert_eq!(compute_health_score(&[issue(Severity::High)]), 75);
        assert_eq!(compute_health_score(&[issue(Severity::Medium)]), 85);
        assert_eq!(compute_health_score(&[issue(Severity::Low)]), 95);
    }

    #[test]
    fn two_issues_deduct_both() {
        let issues = vec![issue(Severity::High), issue(Severity::Medium)];
        assert_eq!(compute_health_score(&issues), 60);
    }

    #[test]
    fn caps_at_three_most_severe() {
        let issues = vec![
            issue(Severity::Low),
            issue(Severity::Low),
            issue(Severity::Low),
            issue(Severity::Low),
        ];
        // 只扣前 3 个 Low = 15 -> 85（而非 80）
        assert_eq!(compute_health_score(&issues), 85);
    }

    #[test]
    fn floor_at_20() {
        let issues = vec![
            issue(Severity::Critical),
            issue(Severity::Critical),
            issue(Severity::Critical),
            issue(Severity::High),
        ];
        // 3 个 Critical = 120 扣分 -> 低于 0 -> 封底 20
        assert_eq!(compute_health_score(&issues), 20);
    }
}
