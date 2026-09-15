// 诊断领域模型 —— 对齐 ERD §3.10 issues 表 + core-rust diagnose crate。
// 与 client-windows DiagnosisModels.cs 完全一致（含 snake_case 序列化）。

import Foundation

/// 问题类别（ERD §3.10 category，snake_case）。
public enum Category: String, CaseIterable, Codable, Sendable {
    case performance
    case disk
    case memory
    case security

    /// 中文显示名。
    public var display: String {
        switch self {
        case .performance: return "性能"
        case .disk: return "磁盘"
        case .memory: return "内存"
        case .security: return "安全"
        }
    }
}

/// 严重度。Critical > High > Medium > Low。
public enum Severity: Int, CaseIterable, Codable, Sendable {
    case low = 0
    case medium = 1
    case high = 2
    case critical = 3
}

extension Severity: Comparable {
    public static func < (lhs: Severity, rhs: Severity) -> Bool { lhs.rawValue < rhs.rawValue }
}

public extension Severity {
    /// 严重度对应的健康分扣分（与 core-rust health.rs 一致）。
    var healthPenalty: Int {
        switch self {
        case .critical: return 40
        case .high: return 25
        case .medium: return 15
        case .low: return 5
        }
    }

    /// 中文显示名。
    var display: String {
        switch self {
        case .critical: return "严重"
        case .high: return "较高"
        case .medium: return "中等"
        case .low: return "轻微"
        }
    }

    /// 界面用颜色（语义化，勿用于文本对比度判定之外）。
    var colorName: String {
        switch self {
        case .critical: return "severityCritical"
        case .high: return "severityHigh"
        case .medium: return "severityMedium"
        case .low: return "severityLow"
        }
    }
}

/// 检测到的一个问题（ERD §3.10 字段映射）。
public struct Issue: Sendable {
    public var category: Category
    public var severity: Severity
    public var title: String
    public var description: String

    /// 证据（关键指标值），JSON 对象形态，对齐 ERD evidence_json。
    public var evidence: [String: Any]

    public init(
        category: Category,
        severity: Severity,
        title: String,
        description: String = "",
        evidence: [String: Any] = [:]
    ) {
        self.category = category
        self.severity = severity
        self.title = title
        self.description = description
        self.evidence = evidence
    }
}

/// 一次诊断的结果（与 Rust DiagnosisReport 对齐）。
public struct DiagnosisReport: Sendable {
    /// 健康分 0~100（100 为最佳）。
    public var healthScore: Int

    /// 问题列表（按严重度降序）。
    public var issues: [Issue]

    /// 诊断完成时间（Unix 毫秒）。
    public var generatedAtUnixMs: Int64

    /// 会话标识（由上层生成）。
    public var sessionId: String

    public init(healthScore: Int = 100, issues: [Issue] = [], generatedAtUnixMs: Int64 = 0, sessionId: String = "") {
        self.healthScore = healthScore
        self.issues = issues
        self.generatedAtUnixMs = generatedAtUnixMs
        self.sessionId = sessionId
    }
}