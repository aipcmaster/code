import XCTest
@testable import AIPCMasterCore

/// Advisor 推理层（与 core-rust advisor 一致）：过滤、去重、风险升序、根因摘要。
final class AdvisorTests: XCTestCase {
    private let advisor = Advisor()

    func testEmptyAdvice() {
        let advice = advisor.advise(DiagnosisReport(healthScore: 100, issues: []))
        XCTAssertTrue(advice.isEmpty)
        XCTAssertTrue(advice.rootCauseSummary.contains("未发现"))
    }

    func testLowSeverityFilteredOut() {
        let report = DiagnosisReport(issues: [Issue(category: .performance, severity: .low, title: "t")])
        XCTAssertTrue(advisor.advise(report).isEmpty)
    }

    func testMemoryHighGivesCleanMemoryLowRisk() {
        let report = DiagnosisReport(issues: [Issue(category: .memory, severity: .high, title: "内存占用偏高")])
        let advice = advisor.advise(report)
        XCTAssertEqual(advice.suggestions.count, 1)
        let s = advice.suggestions[0]
        XCTAssertEqual(s.actionType, .cleanMemory)
        XCTAssertEqual(s.riskLevel, .low)
        XCTAssertFalse(s.requiresConfirm)
    }

    func testMemoryCriticalRequiresConfirm() {
        let report = DiagnosisReport(issues: [Issue(category: .memory, severity: .critical, title: "内存严重偏高")])
        let s = advisor.advise(report).suggestions[0]
        XCTAssertEqual(s.riskLevel, .medium)
        XCTAssertTrue(s.requiresConfirm)
    }

    func testDiskCriticalHighRiskConfirm() {
        let report = DiagnosisReport(issues: [Issue(category: .disk, severity: .critical, title: "磁盘即将占满")])
        let s = advisor.advise(report).suggestions[0]
        XCTAssertEqual(s.actionType, .disk)
        XCTAssertEqual(s.riskLevel, .high)
        XCTAssertTrue(s.requiresConfirm)
    }

    func testPerformanceGivesStartup() {
        let report = DiagnosisReport(issues: [Issue(category: .performance, severity: .high, title: "CPU 偏高")])
        let s = advisor.advise(report).suggestions[0]
        XCTAssertEqual(s.actionType, .startup)
        XCTAssertEqual(s.riskLevel, .low)
    }

    func testSuggestionsSortedByRiskAscending() {
        let report = DiagnosisReport(issues: [
            Issue(category: .disk, severity: .critical, title: "盘满"),
            Issue(category: .memory, severity: .medium, title: "内存轻高"),
            Issue(category: .performance, severity: .high, title: "CPU 高"),
        ])
        let suggestions = advisor.advise(report).suggestions
        for i in 1..<suggestions.count {
            XCTAssertLessThanOrEqual(suggestions[i - 1].riskLevel, suggestions[i].riskLevel,
                                     "建议应按风险升序：\(suggestions)")
        }
    }

    func testDedupMergesSameAction() {
        // 两个内存问题 → 去重为一个 clean_memory，保留更高风险并合并原因
        let report = DiagnosisReport(issues: [
            Issue(category: .memory, severity: .critical, title: "内存严重"),
            Issue(category: .memory, severity: .medium, title: "内存轻高"),
        ])
        let suggestions = advisor.advise(report).suggestions
        XCTAssertEqual(suggestions.count, 1)
        let s = suggestions[0]
        XCTAssertEqual(s.riskLevel, .medium)
        XCTAssertTrue(s.requiresConfirm)
        XCTAssertTrue(s.reason.contains("；"), "两个问题原因应合并：\(s.reason)")
    }

    func testSummaryMentionsCount() {
        let report = DiagnosisReport(issues: [Issue(category: .disk, severity: .critical, title: "盘满")])
        let summary = advisor.advise(report).rootCauseSummary
        XCTAssertTrue(summary.contains("健康分"))
        XCTAssertTrue(summary.contains("1 项优化"))
    }
}