// 健康分计算 —— 精确移植 core-rust crates/aipcmaster-diagnose/src/health.rs。
//
// 算法：
// - 无问题 → 100；
// - 取严重度最高的最多 **3** 个问题扣分（防止几十个 Low 把分打到 0）；
// - 扣分：Critical 40 / High 25 / Medium 15 / Low 5；
// - 下限 20：症状层面的扣分不会打到 0，为后续 AI 根因层留出低分区间。

namespace AIPCMaster.Core.Diagnosis;

public static class HealthScore
{
    /// 由问题列表计算健康分（0~100）。
    public static int Compute(IReadOnlyList<Issue> issues)
    {
        if (issues.Count == 0)
        {
            return 100;
        }

        // 严重度降序，取前 3
        var sorted = issues
            .OrderByDescending(i => i.Severity)
            .Take(3)
            .ToList();

        int penalty = sorted.Sum(i => i.Severity.HealthPenalty());

        return Math.Clamp(100 - penalty, 20, 100);
    }
}