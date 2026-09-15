// macOS 采集后端：纯系统调用（Darwin/Mach），零第三方依赖。
//
// 设计约束：不依赖任何 SPM 外部包（离线可构建），因此不用第三方库，
// 而用系统 API —— host_processor_info（CPU tick 双采样差分，与 Windows
// GetSystemTimes 同构）、host_statistics64（内存）、getloadavg（负载，macOS
// 有！不同于 Windows 恒 nil）、statvfs（磁盘容量）。
//
// CPU 温度在 macOS 上需 SMC/root（powermetrics），V1 恒 nil → 温度规则自动跳过。
//
// 注意（踩坑护栏）：
// - processor 数组长度为 cpuCount × CPU_STATE_MAX 个 integer_t；
// - CPU tick 是 UInt32 计数，可能超过 Int32 上限（忙碌 60+ 小时后），
//   必须按 UInt32 读取再转 UInt64 做差分，防止负数回绕；
// - xsw_usage 是系统头文件已声明的 C 结构，勿自定义同名类型。

import Foundation
import Darwin

/// macOS 采集后端。
public final class MacSystemCollector {
    private let lock = NSLock()

    // CPU 双采样差分状态（与 WindowsSystemCollector 同构）
    private var prevIdleTicks: UInt64 = 0
    private var prevTotalTicks: UInt64 = 0
    private var prevPerCoreIdle: [UInt64] = []
    private var prevPerCoreTotal: [UInt64] = []

    public init() {}

    public func collect() -> SystemSnapshot { collectCore() }

    public func collectFull() -> SystemSnapshot { collectCore() }

    private func collectCore() -> SystemSnapshot {
        SystemSnapshot(
            timestampUnixMs: Int64(Date().timeIntervalSince1970 * 1000),
            cpu: readCpu(),
            memory: readMemory(),
            disks: readDisks()
        )
    }

    // MARK: - CPU（host_processor_info，每核 tick 差分）

    private func readCpu() -> CpuMetrics? {
        var cpuCount: natural_t = 0
        // processor_info_array_t 本身就是 UnsafeMutablePointer<integer_t>?，
        // 声明为不可空 optional 类型以匹配 C 的 out 参数（经典用法，勿嵌套 optional）。
        var info: processor_info_array_t = nil
        var infoCount: mach_msg_type_number_t = 0

        let kr = host_processor_info(
            mach_host_self(),
            PROCESSOR_CPU_LOAD_INFO,
            &cpuCount,
            &info,
            &infoCount
        )

        guard kr == KERN_SUCCESS, let raw = info, cpuCount > 0 else {
            // 读取失败：返回最小可用快照，让诊断规则自然跳过
            return CpuMetrics(coreCount: ProcessInfo.processInfo.activeProcessorCount)
        }

        defer {
            vm_deallocate(
                mach_task_self_,
                vm_address_t(UInt(bitPattern: UnsafeMutableRawPointer(raw))),
                vm_size_t(infoCount) * vm_size_t(MemoryLayout<integer_t>.size)
            )
        }

        // 数组按 CPU 逐个排布，每个 CPU CPU_STATE_MAX 个 tick（USER/SYSTEM/IDLE/NICE）。
        // 用局部常量而非 CPU_STATE_USER 等宏，避免对 C 宏导入的依赖。
        let userIdx = 0, systemIdx = 1, idleIdx = 2, niceIdx = 3
        let statesPerCore = 4
        let coreCount = Int(cpuCount)

        var idleTotals: [UInt64] = []
        var totalTotals: [UInt64] = []

        // 越界护栏：只读 infoCount 范围内（正常情况下 infoCount == cpuCount × 4）
        let readableCores = min(coreCount, Int(infoCount) / statesPerCore)

        let ticks = UnsafeRawPointer(raw).assumingMemoryBound(to: UInt32.self)
        for core in 0..<readableCores {
            let base = core * statesPerCore
            let user = UInt64(ticks[base + userIdx])
            let system = UInt64(ticks[base + systemIdx])
            let idle = UInt64(ticks[base + idleIdx])
            let nice = UInt64(ticks[base + niceIdx])
            idleTotals.append(idle)
            totalTotals.append(user + system + idle + nice)
        }

        var usage = 0.0
        var perCore: [Double] = []

        lock.lock()
        let idleNow = idleTotals.reduce(0, +)
        let totalNow = totalTotals.reduce(0, +)

        if prevTotalTicks != 0 {
            let idleDelta = idleNow - prevIdleTicks
            let totalDelta = totalNow - prevTotalTicks
            if totalDelta > 0 {
                usage = min(100.0, max(0.0, 100.0 * (1.0 - Double(idleDelta) / Double(totalDelta))))
            }

            if prevPerCoreTotal.count == totalTotals.count {
                for i in 0..<totalTotals.count {
                    let tDelta = totalTotals[i] - prevPerCoreTotal[i]
                    let iDelta = idleTotals[i] - prevPerCoreIdle[i]
                    if tDelta > 0 {
                        perCore.append(min(100.0, max(0.0, 100.0 * (1.0 - Double(iDelta) / Double(tDelta)))))
                    } else {
                        perCore.append(0)
                    }
                }
            }
        } else {
            perCore = Array(repeating: 0, count: readableCores)
        }

        prevIdleTicks = idleNow
        prevTotalTicks = totalNow
        prevPerCoreIdle = idleTotals
        prevPerCoreTotal = totalTotals
        lock.unlock()

        // 1/5/15 分钟负载（macOS 真实采集；失败则 nil → 负载规则跳过）
        var loads = [Double](repeating: 0, count: 3)
        let n = getloadavg(&loads, 3)

        return CpuMetrics(
            usagePercent: usage,
            perCoreUsagePercent: perCore,
            loadAvg1: n >= 1 ? loads[0] : nil,
            loadAvg5: n >= 2 ? loads[1] : nil,
            loadAvg15: n >= 3 ? loads[2] : nil,
            coreCount: max(1, readableCores),
            temperatureC: nil // SMC 需特权，V1 不采集（规则跳过）
        )
    }

    // MARK: - 内存（host_statistics64 + vm.swapusage）

    private func readMemory() -> MemoryMetrics? {
        let total = physicalMemoryBytes
        guard total > 0 else { return nil }

        var stats = vm_statistics64_data_t()
        var count = mach_msg_type_number_t(
            MemoryLayout<vm_statistics64_data_t>.size / MemoryLayout<integer_t>.size
        )

        let kr = withUnsafeMutablePointer(to: &stats) { ptr in
            ptr.withMemoryRebound(to: integer_t.self, capacity: Int(count)) { intPtr in
                host_statistics64(mach_host_self(), HOST_VM_INFO64, intPtr, &count)
            }
        }
        guard kr == KERN_SUCCESS else { return nil }

        let pageSize = Double(pageSizeBytes)
        let free = Double(stats.free_count) * pageSize
        let active = Double(stats.active_count) * pageSize
        let wired = Double(stats.wire_count) * pageSize
        let compressed = Double(stats.compressor_page_count) * pageSize

        // 口径与 Activity Monitor 近似：占用 = 活跃 + 线缆 + 压缩
        let used = min(Double(total), active + wired + compressed)
        let available = Double(total) - used
        let usage = min(100.0, max(0.0, used / Double(total) * 100.0))

        // Swap（vm.swapusage；无 swap 的现代 macOS 上 total=0 → swap 规则自动跳过）
        var swap = xsw_usage()
        var swapSize = MemoryLayout<xsw_usage>.size
        let swapOk = sysctlbyname("vm.swapusage", &swap, &swapSize, nil, 0) == 0
        let swapTotal = swapOk ? swap.xsw_total : 0
        let swapUsed = swapOk ? swap.xsw_used : 0

        return MemoryMetrics(
            totalBytes: total,
            usedBytes: UInt64(used),
            availableBytes: UInt64(available),
            usagePercent: usage,
            swapTotalBytes: swapTotal,
            swapUsedBytes: swapUsed
        )
    }

    // MARK: - 磁盘（statvfs 遍历挂载卷，只报容量卷）

    private func readDisks() -> [DiskMetrics] {
        let fm = FileManager.default
        guard let urls = fm.mountedVolumeURLs(
            includingResourceValuesForKeys: [.volumeIsLocalKey, .volumeIsReadOnlyKey],
            options: [.skipHiddenVolumes]
        ) else {
            return []
        }

        var result: [DiskMetrics] = []
        for url in urls {
            guard let values = try? url.resourceValues(forKeys: [.volumeIsLocalKey, .volumeIsReadOnlyKey]),
                  values.volumeIsLocal == true,
                  values.volumeIsReadOnly != true else {
                continue
            }
            let path = url.path
            guard isUserVisibleVolume(path), let m = statvfsInfo(path) else {
                continue
            }
            result.append(m)
        }
        return result
    }

    /// 只看系统可见的数据卷：根 "/" 与 /Volumes 下的用户卷；跳过系统内部卷。
    private func isUserVisibleVolume(_ path: String) -> Bool {
        if path == "/" { return true }
        if path.hasPrefix("/System/Volumes/") { return false }
        if path.hasPrefix("/Volumes/") {
            let name = (path as NSString).lastPathComponent
            let system = ["Recovery", "Preboot", "VM", "Update", "macOS Base System", "Ghost"]
            return !system.contains(name)
        }
        return false
    }

    private func statvfsInfo(_ path: String) -> DiskMetrics? {
        var s = statvfs()
        guard statvfs(path, &s) == 0 else { return nil }

        let frsize = UInt64(s.f_frsize)
        let total = UInt64(s.f_blocks) * frsize
        let available = UInt64(s.f_bavail) * frsize
        let used = total > available ? total - available : 0
        let usage = total == 0 ? 0.0 : min(100.0, max(0.0, Double(used) / Double(total) * 100.0))

        return DiskMetrics(
            device: path,
            mountPoint: path,
            fsType: String(cString: &s.f_fstypename.0),
            totalBytes: total,
            usedBytes: used,
            availableBytes: available,
            usagePercent: usage
        )
    }

    // MARK: - sysctl 工具

    private var physicalMemoryBytes: UInt64 {
        var value: UInt64 = 0
        var size = MemoryLayout<UInt64>.size
        return sysctlbyname("hw.memsize", &value, &size, nil, 0) == 0 ? value : 0
    }

    private var pageSizeBytes: UInt64 {
        var value: UInt64 = 0
        var size = MemoryLayout<UInt64>.size
        if sysctlbyname("hw.pagesize", &value, &size, nil, 0) == 0, value > 0 {
            return value
        }
        return 4096
    }
}