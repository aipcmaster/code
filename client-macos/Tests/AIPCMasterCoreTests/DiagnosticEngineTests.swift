import XCTest
@testable import AIPCMasterCore

/// 诊断规则阈值（与 core-rust snapshot_rules.rs 一致），
/// 以及「字段缺失自动跳过」的 Option 同构语义。
final class DiagnosticEngineTests: XCTestCase {
    private let engine = DiagnosticEngine()

    func testCpuHigh() {
        let snap = SystemSnapshot(cpu: CpuMetrics(usagePercent: 95, coreCount: 8, loadAvg1: 1))
        let issues = engine.diagnose(snap).issues
        XCTAssertTrue(issues.contains { $0.category == .performance && $0.severity == .high })
    }

    func testCpuNormalNoIssue() {
        let snap = SystemSnapshot(cpu: CpuMetrics(usagePercent: 30, coreCount: 8, loadAvg1: 1))
        XCTAssertTrue(engine.diagnose(snap).issues.isEmpty)
    }

    func testMemoryHigh() {
        let snap = SystemSnapshot(memory: MemoryMetrics(totalBytes: 100, usedBytes: 92, availableBytes: 8, usagePercent: 92))
        XCTAssertTrue(engine.diagnose(snap).issues.contains { $0.category == .memory && $0.severity == .high })
    }

    func testMemoryMedium() {
        let snap = SystemSnapshot(memory: MemoryMetrics(totalBytes: 100, usedBytes: 78, availableBytes: 22, usagePercent: 78))
        XCTAssertTrue(engine.diagnose(snap).issues.contains { $0.category == .memory && $0.severity == .medium })
    }

    func testSwapHalfIsMedium() {
        let snap = SystemSnapshot(memory: MemoryMetrics(
            totalBytes: 100, usedBytes: 50, availableBytes: 50, usagePercent: 50,
            swapTotalBytes: 100, swapUsedBytes: 60))
        XCTAssertTrue(engine.diagnose(snap).issues.contains { $0.category == .memory && $0.severity == .medium })
    }

    func testNoSwapSkipsRule() {
        let snap = SystemSnapshot(memory: MemoryMetrics(
            totalBytes: 100, usedBytes: 90, availableBytes: 10, usagePercent: 90,
            swapTotalBytes: 0, swapUsedBytes: 0))
        XCTAssertFalse(engine.diagnose(snap).issues.contains { $0.title.contains("Swap") })
    }

    func testDiskCritical() {
        let snap = SystemSnapshot(disks: [DiskMetrics(device: "/", mountPoint: "/", totalBytes: 100, usedBytes: 96, availableBytes: 4, usagePercent: 96)])
        XCTAssertTrue(engine.diagnose(snap).issues.contains { $0.category == .disk && $0.severity == .critical })
    }

    func testDiskHigh() {
        let snap = SystemSnapshot(disks: [DiskMetrics(device: "/", mountPoint: "/", totalBytes: 100, usedBytes: 88, availableBytes: 12, usagePercent: 88)])
        XCTAssertTrue(engine.diagnose(snap).issues.contains { $0.category == .disk && $0.severity == .high })
    }

    func testDiskReportsWorstOnly() {
        let snap = SystemSnapshot(disks: [
            DiskMetrics(device: "/", mountPoint: "/", totalBytes: 100, usedBytes: 90, availableBytes: 10, usagePercent: 90),
            DiskMetrics(device: "/Volumes/Data", mountPoint: "/Volumes/Data", totalBytes: 100, usedBytes: 40, availableBytes: 60, usagePercent: 40),
        ])
        let issues = engine.diagnose(snap).issues
        XCTAssertEqual(issues.filter { $0.category == .disk }.count, 1)
        XCTAssertEqual(issues.filter { $0.category == .disk }.first?.severity, .high)
    }

    func testLoadHigh() {
        let snap = SystemSnapshot(cpu: CpuMetrics(usagePercent: 10, coreCount: 8, loadAvg1: 20))
        XCTAssertTrue(engine.diagnose(snap).issues.contains { $0.category == .performance && $0.severity == .medium })
    }

    func testLoadBelowThresholdNoIssue() {
        let snap = SystemSnapshot(cpu: CpuMetrics(usagePercent: 10, coreCount: 8, loadAvg1: 8))
        XCTAssertFalse(engine.diagnose(snap).issues.contains { $0.title.contains("繁忙") })
    }

    func testNilFieldsSkipAllRules() {
        // 空快照：所有规则应静默跳过（Option 同构），健康分 100
        let report = engine.diagnose(SystemSnapshot(timestampUnixMs: 1))
        XCTAssertEqual(report.healthScore, 100)
        XCTAssertTrue(report.issues.isEmpty)
    }

    func testTemperatureNilSkips() {
        // macOS 温度恒 nil → 不触发温度规则（不误报）
        let snap = SystemSnapshot(cpu: CpuMetrics(usagePercent: 20, coreCount: 8, temperatureC: nil))
        XCTAssertFalse(engine.diagnose(snap).issues.contains { $0.title.contains("温度") })
    }

    func testScoreComputedFromSorted() {
        let snap = SystemSnapshot(disks: [DiskMetrics(device: "/", mountPoint: "/", totalBytes: 100, usedBytes: 96, availableBytes: 4, usagePercent: 96)])
        XCTAssertEqual(engine.diagnose(snap).healthScore, 60) // 100 - 40
    }

    func testIssuesSortedBySeverityDesc() {
        let snap = SystemSnapshot(
            cpu: CpuMetrics(usagePercent: 95, coreCount: 8, loadAvg1: 20),
            memory: MemoryMetrics(totalBytes: 100, usedBytes: 92, availableBytes: 8, usagePercent: 92))
        let issues = engine.diagnose(snap).issues
        for i in 1..<issues.count {
            XCTAssertGreaterThanOrEqual(issues[i - 1].severity, issues[i].severity)
        }
    }
}