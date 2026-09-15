// 诊断引擎 —— 精确移植 core-rust snapshot_rules.rs（阈值 V1 经验值）：
// - CPU 使用率 > 90%                → High（Performance）
// - 内存使用率 > 90% → High；> 75%  → Medium（Memory）
// - 交换分区使用率 > 50%            → Medium（Memory）
// - 磁盘使用率 > 95% → Critical；> 85% → High（Disk）
// - CPU 温度 > 95°C → Critical；> 80°C → High（Performance）
// - 1 分钟负载 > 核数 × 2           → Medium（Performance）
//
// 与 Rust 语义一致：快照字段为 nil / 空时规则自动跳过（Option 同构）。
// macOS 温度采集不到 → 温度规则天然不触发（不误报）；负载是真实值 → 负载规则生效。

import Foundation

public final class DiagnosticEngine {
    public init() {}

    /// 执行全部标准规则，返回按严重度降序的问题列表，并计算健康分。
    public func diagnose(_ snap: SystemSnapshot, sessionId: String = "local") -> DiagnosisReport {
        var issues: [Issue] = []

        cpuUsage(snap, into: &issues)
        memoryUsage(snap, into: &issues)
        swapUsage(snap, into: &issues)
        diskUsage(snap, into: &issues)
        cpuTemperature(snap, into: &issues)
        loadAverage(snap, into: &issues)

        let sorted = issues.sorted { $0.severity > $1.severity }

        return DiagnosisReport(
            healthScore: HealthScore.compute(sorted),
            issues: sorted,
            generatedAtUnixMs: snap.timestampUnixMs,
            sessionId: sessionId
        )
    }

    // MARK: - 规则实现（阈值与 core-rust / client-windows 完全一致）

    private func cpuUsage(_ s: SystemSnapshot, into issues: inout [Issue]) {
        guard let cpu = s.cpu else { return }

        if cpu.usagePercent >= 90.0 {
            issues.append(Issue(
                category: .performance,
                severity: .high,
                title: "CPU 使用率持续偏高",
                description: "CPU 平均使用率超过 90%，可能由后台任务或异常进程导致，建议排查占用最高的进程。",
                evidence: ["cpu_percent": round1(cpu.usagePercent)]
            ))
        }
    }

    private func memoryUsage(_ s: SystemSnapshot, into issues: inout [Issue]) {
        guard let mem = s.memory else { return }

        if mem.usagePercent >= 90.0 {
            issues.append(Issue(
                category: .memory,
                severity: .high,
                title: "内存占用偏高",
                description: "内存使用率超过 90%，可能出现卡顿，建议清理后台进程或检查内存泄漏。",
                evidence: ["mem_percent": round1(mem.usagePercent)]
            ))
        } else if mem.usagePercent >= 75.0 {
            issues.append(Issue(
                category: .memory,
                severity: .medium,
                title: "内存占用偏高（轻度）",
                description: "内存使用率超过 75%，建议关注内存大户。",
                evidence: ["mem_percent": round1(mem.usagePercent)]
            ))
        }
    }

    private func swapUsage(_ s: SystemSnapshot, into issues: inout [Issue]) {
        guard let mem = s.memory, mem.swapTotalBytes > 0 else { return }

        let used = 100.0 * Double(mem.swapUsedBytes) / Double(mem.swapTotalBytes)
        if used >= 50.0 {
            issues.append(Issue(
                category: .memory,
                severity: .medium,
                title: "交换分区（Swap）使用过半",
                description: "Swap 使用率超过 50%，物理内存可能不足，频繁换页会造成卡顿。",
                evidence: ["swap_percent": round1(used)]
            ))
        }
    }

    private func diskUsage(_ s: SystemSnapshot, into issues: inout [Issue]) {
        // 只报最满的一块容量盘
        guard let worst = s.disks.max(by: { $0.usagePercent < $1.usagePercent }) else { return }

        let p = worst.usagePercent
        if p >= 95.0 {
            issues.append(Issue(
                category: .disk,
                severity: .critical,
                title: "磁盘空间即将占满",
                description: "磁盘使用率超过 95%，继续使用将导致系统异常，建议立即清理。",
                evidence: ["device": worst.device, "percent": round1(p)]
            ))
        } else if p >= 85.0 {
            issues.append(Issue(
                category: .disk,
                severity: .high,
                title: "磁盘空间不足",
                description: "磁盘使用率超过 85%，建议清理缓存和大文件。",
                evidence: ["device": worst.device, "percent": round1(p)]
            ))
        }
    }

    private func cpuTemperature(_ s: SystemSnapshot, into issues: inout [Issue]) {
        guard let temp = s.cpu?.temperatureC else { return }

        if temp >= 95.0 {
            issues.append(Issue(
                category: .performance,
                severity: .critical,
                title: "CPU 温度过高",
                description: "CPU 温度超过 95°C，可能降频或损坏硬件，请检查散热。",
                evidence: ["temp_c": round1(temp)]
            ))
        } else if temp >= 80.0 {
            issues.append(Issue(
                category: .performance,
                severity: .high,
                title: "CPU 温度偏高",
                description: "CPU 温度超过 80°C，建议清理风扇或降低负载。",
                evidence: ["temp_c": round1(temp)]
            ))
        }
    }

    private func loadAverage(_ s: SystemSnapshot, into issues: inout [Issue]) {
        guard let load = s.cpu?.loadAvg1, load >= 0 else { return }

        let cores = max(s.cpu?.coreCount ?? 1, 1)
        if load >= Double(cores) * 2.0 {
            issues.append(Issue(
                category: .performance,
                severity: .medium,
                title: "系统整体繁忙",
                description: "1 分钟平均负载达到核心数的 2 倍以上，系统可能接近过载。",
                evidence: ["load_avg_1": round1(load), "cores": cores]
            ))
        }
    }

    /// 保留 1 位小数（与 C# "F1" 一致）。
    private func round1(_ v: Double) -> Double {
        (v * 10).rounded() / 10
    }
}