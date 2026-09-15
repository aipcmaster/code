import XCTest
@testable import AIPCMasterCore

/// 健康分算法（移植 core-rust health.rs 已知用例，与 Windows ConsoleSelfCheck 一致）。
final class HealthScoreTests: XCTestCase {
    func testEmptyIs100() {
        XCTAssertEqual(HealthScore.compute([]), 100)
    }

    func testSingleCriticalIs60() {
        XCTAssertEqual(HealthScore.compute([issue(.critical)]), 60)
    }

    func testHighPlusMediumIs60() {
        // 100 - 25 - 15 = 60
        XCTAssertEqual(HealthScore.compute([issue(.high), issue(.medium)]), 60)
    }

    func testFourLowsTakeTop3Is85() {
        // 取前 3 个 Low：100 - 5×3 = 85（防止几十个 Low 把分打到 0）
        let lows = (0..<4).map { _ in issue(.low) }
        XCTAssertEqual(HealthScore.compute(lows), 85)
    }

    func testFloorIs20() {
        // 3 × Critical = 120 扣分 → 封底 20
        let floods = (0..<4).map { _ in issue(.critical) }
        XCTAssertEqual(HealthScore.compute(floods), 20)
    }

    private func issue(_ severity: Severity) -> Issue {
        Issue(category: .performance, severity: severity, title: "t")
    }
}