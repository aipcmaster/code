// AIPCMaster（AI电脑大师）Windows 客户端 —— 采集数据模型
//
// 与 core-rust crates/aipcmaster-collect/src/metrics.rs 一一对应（跨平台共享建模）。
// 字段均为可空以表达「采集不到」的语义：规则对 null 字段自动跳过（与 Rust Option 同构）。

namespace AIPCMaster.Core.Collectors;

/// CPU 指标。
public sealed class CpuMetrics
{
    /// 整体使用率 0~100。
    public double UsagePercent { get; set; }

    /// 每核使用率。
    public List<double> PerCoreUsagePercent { get; set; } = [];

    /// Windows 无 loadavg；保留字段以便未来接入 PDH 队列长度（null = 不评估负载规则）。
    public double? LoadAvg1 { get; set; }

    public double? LoadAvg5 { get; set; }

    public double? LoadAvg15 { get; set; }

    public int CoreCount { get; set; }

    /// CPU 温度（°C）。Windows 需 WMI/驱动，无则为 null（跳过温度规则）。
    public double? TemperatureC { get; set; }
}

/// 内存指标。
public sealed class MemoryMetrics
{
    public ulong TotalBytes { get; set; }
    public ulong UsedBytes { get; set; }
    public ulong AvailableBytes { get; set; }
    public double UsagePercent { get; set; }
    public ulong SwapTotalBytes { get; set; }
    public ulong SwapUsedBytes { get; set; }
}

/// 磁盘容量指标。
public sealed class DiskMetrics
{
    public string Device { get; set; } = "";
    public string MountPoint { get; set; } = "";
    public string FsType { get; set; } = "";
    public ulong TotalBytes { get; set; }
    public ulong UsedBytes { get; set; }
    public ulong AvailableBytes { get; set; }
    public double UsagePercent { get; set; }
}

/// 一次系统快照（与 Rust SystemSnapshot 对齐）。
public sealed class SystemSnapshot
{
    public long TimestampUnixMs { get; set; }
    public CpuMetrics? Cpu { get; set; }
    public MemoryMetrics? Memory { get; set; }
    public List<DiskMetrics> Disks { get; set; } = [];
}