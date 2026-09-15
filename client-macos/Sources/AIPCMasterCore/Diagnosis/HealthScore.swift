// 健康分计算 —— 精确移植 core-rust health.rs（与 client-windows HealthScore.cs 一致）。
//
// 算法：
// - 无问题 → 100；
// - 取严重度最高的最多 **3** 个问题扣分（防止几十个 Low 把分打到 0）；
// - 扣分：Critical 40 / High 25 / Medium 15 / Low 5；
// - 下限 20：症状层面的扣分不会打到 0，为后续 AI 根因层留出低分区间。

import Foundation

public enum HealthScore {
    /// 由问题列表计算健康分（0~100）。
    public static func compute(_ issues: [Issue]) -> Int {
        if issues.isEmpty {
            return 100
        }

        // 严重度降序，取前 3
        let sorted = issues.sorted { $0.severity > $1.severity }.prefix(3)
        let penalty = sorted.reduce(0) { $0 + $1.severity.healthPenalty }

        return min(100, max(20, 100 - penalty))
    }
}