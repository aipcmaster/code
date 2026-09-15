// AIPCMaster（AI电脑大师）macOS 客户端 —— 采集数据模型
//
// 与 client-windows AIPCMaster.Core/Collectors/SystemSnapshot.cs 一一对应
// （跨平台共享建模，最终语义对齐 core-rust metrics.rs）。
// 字段尽量可空，表达「采集不到」：诊断规则对 nil 字段自动跳过（与 Rust Option 同构）。

import Foundation

/// CPU 指标。
public struct CpuMetrics {
    /// 整体使用率 0~100。
    public var usagePercent: Double

    /// 每核使用率 0~100（macOS 由 host_processor_info 每核 tick 差分得到）。
    public var perCoreUsagePercent: [Double]

    /// 1 / 5 / 15 分钟平均负载（macOS 有 getloadavg，是真实值，非 Windows 的 nil）。
    public var loadAvg1: Double?
    public var loadAvg5: Double?
    public var loadAvg15: Double?

    /// 逻辑核心数。
    public var coreCount: Int

    /// CPU 温度（°C）。macOS 需 SMC/root 权限，V1 恒为 nil → 温度规则自动跳过（不误报）。
    public var temperatureC: Double?

    public init(
        usagePercent: Double = 0,
        perCoreUsagePercent: [Double] = [],
        loadAvg1: Double? = nil,
        loadAvg5: Double? = nil,
        loadAvg15: Double? = nil,
        coreCount: Int = 1,
        temperatureC: Double? = nil
    ) {
        self.usagePercent = usagePercent
        self.perCoreUsagePercent = perCoreUsagePercent
        self.loadAvg1 = loadAvg1
        self.loadAvg5 = loadAvg5
        self.loadAvg15 = loadAvg15
        self.coreCount = coreCount
        self.temperatureC = temperatureC
    }
}

/// 内存指标（字节）。
public struct MemoryMetrics {
    public var totalBytes: UInt64
    public var usedBytes: UInt64
    public var availableBytes: UInt64
    public var usagePercent: Double
    public var swapTotalBytes: UInt64
    public var swapUsedBytes: UInt64

    public init(
        totalBytes: UInt64 = 0,
        usedBytes: UInt64 = 0,
        availableBytes: UInt64 = 0,
        usagePercent: Double = 0,
        swapTotalBytes: UInt64 = 0,
        swapUsedBytes: UInt64 = 0
    ) {
        self.totalBytes = totalBytes
        self.usedBytes = usedBytes
        self.availableBytes = availableBytes
        self.usagePercent = usagePercent
        self.swapTotalBytes = swapTotalBytes
        self.swapUsedBytes = swapUsedBytes
    }
}

/// 磁盘容量指标。
public struct DiskMetrics {
    /// 设备标识（macOS 用挂载点路径，如 "/"）。
    public var device: String

    /// 挂载点。
    public var mountPoint: String

    /// 文件系统类型（如 apfs / hfs+）。
    public var fsType: String

    public var totalBytes: UInt64
    public var usedBytes: UInt64
    public var availableBytes: UInt64
    public var usagePercent: Double

    public init(
        device: String = "",
        mountPoint: String = "",
        fsType: String = "",
        totalBytes: UInt64 = 0,
        usedBytes: UInt64 = 0,
        availableBytes: UInt64 = 0,
        usagePercent: Double = 0
    ) {
        self.device = device
        self.mountPoint = mountPoint
        self.fsType = fsType
        self.totalBytes = totalBytes
        self.usedBytes = usedBytes
        self.availableBytes = availableBytes
        self.usagePercent = usagePercent
    }
}

/// 一次系统快照（与 Rust / C# SystemSnapshot 对齐）。
public struct SystemSnapshot {
    public var timestampUnixMs: Int64
    public var cpu: CpuMetrics?
    public var memory: MemoryMetrics?
    public var disks: [DiskMetrics]

    public init(
        timestampUnixMs: Int64 = 0,
        cpu: CpuMetrics? = nil,
        memory: MemoryMetrics? = nil,
        disks: [DiskMetrics] = []
    ) {
        self.timestampUnixMs = timestampUnixMs
        self.cpu = cpu
        self.memory = memory
        self.disks = disks
    }
}