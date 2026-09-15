// AI 推理层（规则化引用实现）—— 对齐 ERD §3.11 optimization_actions
// 与 core-rust advisor + client-windows Advisor.cs 完全一致。
//
// 输入诊断报告 → 输出根因摘要 + 优化动作建议（action_type / risk_level /
// requires_confirm / parameters）。高危动作必须用户确认（SD §2.3 默认只读）。

import Foundation

/// 优化动作类型（ERD §3.11 action_type，snake_case）。
public enum ActionType: String, Sendable {
    case cleanMemory = "clean_memory"
    case startup
    case disk
    case generic
}

/// 风险级别（ERD §3.11 risk_level）。
public enum RiskLevel: Int, CaseIterable, Sendable {
    case low = 0
    case medium = 1
    case high = 2
}

extension RiskLevel: Comparable {
    public static func < (lhs: RiskLevel, rhs: RiskLevel) -> Bool { lhs.rawValue < rhs.rawValue }
}

public extension RiskLevel {
    var display: String {
        switch self {
        case .low: return "低风险"
        case .medium: return "中风险"
        case .high: return "高风险"
        }
    }
}

/// 一条优化动作建议（ERD §3.11 字段映射）。
public struct Suggestion: Identifiable, Sendable {
    public var actionType: ActionType
    public var riskLevel: RiskLevel

    /// 高危动作必须用户确认（SD §2.3 默认只读）。
    public var requiresConfirm: Bool

    public var parameters: [String: Any]
    public var reason: String
    public var category: Category

    /// 用于 SwiftUI ForEach（同一动作+类别组合唯一）。
    public var id: String { "\(actionType.rawValue)|\(category.rawValue)" }

    public init(
        actionType: ActionType,
        riskLevel: RiskLevel,
        requiresConfirm: Bool = false,
        parameters: [String: Any] = [:],
        reason: String = "",
        category: Category
    ) {
        self.actionType = actionType
        self.riskLevel = riskLevel
        self.requiresConfirm = requiresConfirm
        self.parameters = parameters
        self.reason = reason
        self.category = category
    }
}

/// 一次推理的结果：根因摘要 + 动作建议列表。
public struct Advice: Sendable {
    public var rootCauseSummary: String
    public var suggestions: [Suggestion]

    public var isEmpty: Bool { suggestions.isEmpty }

    public init(rootCauseSummary: String = "", suggestions: [Suggestion] = []) {
        self.rootCauseSummary = rootCauseSummary
        self.suggestions = suggestions
    }
}

/// AI 推理引擎（规则化引用实现，纯计算可测，与 core-rust 同构）。
public final class Advisor {
    public init() {}

    public func advise(_ report: DiagnosisReport) -> Advice {
        let suggestions = report.issues
            .filter { $0.severity >= .medium }
            .map(suggestFor)
        let deduped = dedup(suggestions)
        let sorted = deduped.sorted { $0.riskLevel < $1.riskLevel }

        return Advice(
            rootCauseSummary: buildSummary(report, sorted),
            suggestions: sorted
        )
    }

    // MARK: - 建议生成（与 core-rust advisor::suggest_for 对齐）

    private func suggestFor(_ issue: Issue) -> Suggestion {
        switch issue.category {
        case .memory:
            let critical = issue.severity == .critical
            return Suggestion(
                actionType: .cleanMemory,
                riskLevel: critical ? .medium : .low,
                requiresConfirm: critical,
                parameters: ["target": "page_cache", "amount": "auto"],
                reason: critical
                    ? "内存严重偏高，建议清理缓存页面释放内存"
                    : "内存使用率偏高，建议清理缓存页面释放内存",
                category: issue.category
            )

        case .disk:
            let high = issue.severity >= .high
            return Suggestion(
                actionType: .disk,
                riskLevel: high ? .high : .medium,
                requiresConfirm: high,
                parameters: ["target": "temp_files", "min_age_days": 30],
                reason: "磁盘空间紧张，建议清理临时文件与缓存",
                category: issue.category
            )

        case .performance:
            return Suggestion(
                actionType: .startup,
                riskLevel: .low,
                requiresConfirm: false,
                parameters: ["target": "startup_items"],
                reason: "CPU/负载偏高，建议检查开机自启项与后台进程",
                category: issue.category
            )

        case .security:
            return Suggestion(
                actionType: .generic,
                riskLevel: .high,
                requiresConfirm: true,
                parameters: ["target": "security_scan"],
                reason: "存在安全风险，建议执行安全扫描并更新系统",
                category: issue.category
            )
        }
    }

    /// 同类动作去重：保留风险最高的（reason 合并）。
    private func dedup(_ items: [Suggestion]) -> [Suggestion] {
        var map: [String: Suggestion] = [:]
        for item in items {
            let key = "\(item.actionType.rawValue)|\(item.category.rawValue)"
            if let existing = map[key] {
                if item.riskLevel > existing.riskLevel {
                    map[key]?.riskLevel = item.riskLevel
                    map[key]?.requiresConfirm = item.requiresConfirm
                }
                map[key]?.reason = "\(existing.reason)；\(item.reason)"
            } else {
                map[key] = item
            }
        }
        return Array(map.values)
    }

    // MARK: - 根因摘要（与 core-rust build_summary 对齐）

    private func buildSummary(_ report: DiagnosisReport, _ suggestions: [Suggestion]) -> String {
        if suggestions.isEmpty {
            return "系统健康分 \(report.healthScore)，未发现需要优化的问题。"
        }

        guard let worst = report.issues.map(\.severity).max() else {
            return "系统健康分 \(report.healthScore)。"
        }
        let worstWord = worst.display

        let kinds = Set(suggestions.map { $0.actionType.rawValue }).sorted()
        let actionNames = suggestions.map { $0.actionType.rawValue }

        return "健康分 \(report.healthScore)。存在\(worstWord)风险问题（\(kinds.joined(separator: "、"))），"
            + "建议执行 \(suggestions.count) 项优化：\(actionNames.joined(separator: "、"))。"
    }
}