// AIPCMaster macOS 自检程序（无 UI，只读）—— 对应 Windows ConsoleSelfCheck。
// 验证 采集 → 诊断 → 建议 → 健康分算法 全链路，并输出报告。
// 用法：swift run AIPCMasterSelfCheck

import Foundation
import AIPCMasterCore

var passed = 0
var failed = 0

func check(_ name: String, _ ok: Bool, _ detail: String? = nil) {
    if ok {
        passed += 1
        print("  ✅ \(name)")
    } else {
        failed += 1
        print("  ❌ \(name)\(detail.map { "—— \($0)" } ?? "")")
    }
}

print("════════════════════════════════════════")
print(" AIPCMaster 自检 · 采集→诊断→建议 全链路（macOS）")
print(" 时间: \(Date())")
print("════════════════════════════════════════")

// 1. 采集（预热一次 CPU 采样，睡 1s 取真实差分值；只读，不执行任何优化动作）
let collector = MacSystemCollector()
_ = collector.collectFull()
sleep(1)
let snap = collector.collectFull()

print("\n[1/5] 采集")
check("采集返回快照", true)
check("CPU 指标存在", snap.cpu != nil, snap.cpu.map { String(format: "UsagePercent=%.1f%%", $0.usagePercent) } ?? "nil")
check("CPU 使用率在 0-100", snap.cpu.map { $0.usagePercent >= 0 && $0.usagePercent <= 100 } ?? false)
check("核心数 > 0", (snap.cpu?.coreCount ?? 0) > 0)
check("每核指标存在", (snap.cpu?.perCoreUsagePercent.count ?? 0) > 0, "\(snap.cpu?.perCoreUsagePercent.count ?? 0) 核")
check("内存指标存在", snap.memory != nil)
check("内存总量 > 0", (snap.memory?.totalBytes ?? 0) > 0)
check("内存使用率 0-100", snap.memory.map { $0.usagePercent >= 0 && $0.usagePercent <= 100 } ?? false)
print("     CPU \(String(format: "%.1f", snap.cpu?.usagePercent ?? 0))% | "
    + "内存 \(String(format: "%.1f", snap.memory?.usagePercent ?? 0))% | 盘 \(snap.disks.count) 个")

// 2. 诊断
print("\n[2/5] 诊断（规则移植阈值对齐 core-rust）")
let engine = DiagnosticEngine()
let report = engine.diagnose(snap, sessionId: "selfcheck")
check("诊断报告生成", true)
check("健康分 20-100", report.healthScore >= 20 && report.healthScore <= 100, "score=\(report.healthScore)")
print("     健康分 \(report.healthScore)，问题 \(report.issues.count) 项")
for issue in report.issues {
    print("     · [\(issue.severity.rawValue)] \(issue.title)")
}

// 3. 建议
print("\n[3/5] 优化建议（ERD 3.11）")
let advisor = Advisor()
let advice = advisor.advise(report)
check("建议生成", true)
check("根因摘要非空", !advice.rootCauseSummary.isEmpty)
check("建议按风险升序", isSortedByRisk(advice.suggestions))
print("     \(advice.rootCauseSummary)")

// 4. 健康分算法单元校验（移植 core-rust health.rs 已知用例）
print("\n[4/5] 健康分算法（移植对齐）")
let critical = Issue(category: .performance, severity: .critical, title: "t")
let high = Issue(category: .performance, severity: .high, title: "t")
let medium = Issue(category: .performance, severity: .medium, title: "t")
check("空问题=100", HealthScore.compute([]) == 100)
check("单Critical=60", HealthScore.compute([critical]) == 60)
check("High+Medium=60", HealthScore.compute([high, medium]) == 60)
let lows = (0..<4).map { _ in Issue(category: .performance, severity: .low, title: "t") }
check("前3Low取扣=85", HealthScore.compute(lows) == 85)
let floors = (0..<4).map { _ in critical }
check("封底=20", HealthScore.compute(floors) == 20)

// 5. 诊断规则阈值抽查（构造快照 → 期望问题类别）
print("\n[5/5] 诊断规则阈值（构造用例）")
let cpuSnap = SystemSnapshot(cpu: CpuMetrics(usagePercent: 95, coreCount: 8, loadAvg1: 1))
check("CPU>90 → High(Performance)", engine.diagnose(cpuSnap).issues.contains {
    $0.category == .performance && $0.severity == .high
})
let memSnap = SystemSnapshot(memory: MemoryMetrics(totalBytes: 100, usedBytes: 92, availableBytes: 8, usagePercent: 92))
check("内存>90 → High(Memory)", engine.diagnose(memSnap).issues.contains {
    $0.category == .memory && $0.severity == .high
})
let diskSnap = SystemSnapshot(disks: [DiskMetrics(device: "/", mountPoint: "/", fsType: "apfs", totalBytes: 100, usedBytes: 96, availableBytes: 4, usagePercent: 96)])
check("磁盘>95 → Critical(Disk)", engine.diagnose(diskSnap).issues.contains {
    $0.category == .disk && $0.severity == .critical
})
let loadSnap = SystemSnapshot(cpu: CpuMetrics(usagePercent: 10, coreCount: 8, loadAvg1: 20))
check("负载>核×2 → Medium(Performance)", engine.diagnose(loadSnap).issues.contains {
    $0.category == .performance && $0.severity == .medium
})

// 汇总
print("\n════════════════════════════════════════")
print(" 结果: \(passed) 通过, \(failed) 失败")
print("════════════════════════════════════════")
exit(failed == 0 ? 0 : 1)

func isSortedByRisk(_ items: [Suggestion]) -> Bool {
    for i in 1..<items.count where items[i].riskLevel < items[i - 1].riskLevel {
        return false
    }
    return true
}