//! 诊断规则的抽象：一个规则检查一条快照，返回一条问题或空。

use crate::issue::Issue;
use aipcmaster_collect::SystemSnapshot;

/// 规则的判定结果。`None` 表示无问题。
pub type RuleVerdict = Option<Issue>;

/// 诊断规则 trait。新规则只需实现 [`DiagnosticRule::check`]。
pub trait DiagnosticRule: Send + Sync {
    fn check(&self, snap: &SystemSnapshot) -> RuleVerdict;
}

/// 便捷包装：把闭包变成规则。
pub struct FnRule {
    name: String,
    f: Box<dyn Fn(&SystemSnapshot) -> RuleVerdict + Send + Sync>,
}

impl FnRule {
    pub fn new(
        name: impl Into<String>,
        f: impl Fn(&SystemSnapshot) -> RuleVerdict + Send + Sync + 'static,
    ) -> Self {
        Self {
            name: name.into(),
            f: Box::new(f),
        }
    }
}

impl std::fmt::Debug for FnRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FnRule").field("name", &self.name).finish()
    }
}

impl DiagnosticRule for FnRule {
    fn check(&self, snap: &SystemSnapshot) -> RuleVerdict {
        (self.f)(snap)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fn_rule_wrapper() {
        let rule = FnRule::new("always-nothing", |_| None);
        let snap = SystemSnapshot::default();
        assert!(rule.check(&snap).is_none());
    }
}
