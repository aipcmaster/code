// 诊断引擎 —— 精确移植 core-rust snapshot_rules.rs（阈值 V1 经验值）：
// - CPU 使用率 > 90%                → High（Performance）
// - 内存使用率 > 90% → High；> 75%  → Medium（Memory）
// - 交换分区使用率 > 50%            → Medium（Memory）
// - 磁盘使用率 > 95% → Critical；> 85% → High（Disk）
// - CPU 温度 > 95°C → Critical；> 80°C → High（Performance）
// - 1 分钟负载 > 核数 × 2           → Medium（Performance）
//
// 与 Rust 语义完全一致：快照字段为 null / 空时规则自动跳过（Option 同构）。
// Windows 后端 CPU 温度与 loadavg 恒为 null → 对应规则天然不触发（不误报）。

using System.Text.Json;
using AIPCMaster.Core.Collectors;

namespace AIPCMaster.Core.Diagnosis;

public sealed class DiagnosticEngine
{
    /// 执行全部标准规则，返回按严重度降序的问题列表，并计算健康分。
    public DiagnosisReport Diagnose(SystemSnapshot snap, string sessionId)
    {
        var issues = new List<Issue>();

        CpuUsage(snap, issues);
        MemoryUsage(snap, issues);
        SwapUsage(snap, issues);
        DiskUsage(snap, issues);
        CpuTemperature(snap, issues);
        LoadAverage(snap, issues);

        var sorted = issues
            .OrderByDescending(i => i.Severity)
            .ToList();

        return new DiagnosisReport
        {
            HealthScore = HealthScore.Compute(sorted),
            Issues = sorted,
            GeneratedAtUnixMs = snap.TimestampUnixMs,
            SessionId = sessionId,
        };
    }

    // ---- 规则实现（阈值与 core-rust 完全一致） ----

    private static void CpuUsage(SystemSnapshot s, List<Issue> issues)
    {
        var cpu = s.Cpu;
        if (cpu is null)
        {
            return;
        }

        if (cpu.UsagePercent >= 90.0)
        {
            issues.Add(new Issue
            {
                Category = Category.Performance,
                Severity = Severity.High,
                Title = "CPU 使用率持续偏高",
                Description = "CPU 平均使用率超过 90%，可能由后台任务或异常进程导致，建议排查占用最高的进程。",
                Evidence = JsonDocument.Parse($$"""{"cpu_percent": {{cpu.UsagePercent.ToString("F1")}}}""").RootElement,
            });
        }
    }

    private static void MemoryUsage(SystemSnapshot s, List<Issue> issues)
    {
        var mem = s.Memory;
        if (mem is null)
        {
            return;
        }

        if (mem.UsagePercent >= 90.0)
        {
            issues.Add(new Issue
            {
                Category = Category.Memory,
                Severity = Severity.High,
                Title = "内存占用偏高",
                Description = "内存使用率超过 90%，可能出现卡顿，建议清理后台进程或检查内存泄漏。",
                Evidence = JsonDocument.Parse($$"""{"mem_percent": {{mem.UsagePercent.ToString("F1")}}}""").RootElement,
            });
        }
        else if (mem.UsagePercent >= 75.0)
        {
            issues.Add(new Issue
            {
                Category = Category.Memory,
                Severity = Severity.Medium,
                Title = "内存占用偏高（轻度）",
                Description = "内存使用率超过 75%，建议关注内存大户。",
                Evidence = JsonDocument.Parse($$"""{"mem_percent": {{mem.UsagePercent.ToString("F1")}}}""").RootElement,
            });
        }
    }

    private static void SwapUsage(SystemSnapshot s, List<Issue> issues)
    {
        var mem = s.Memory;
        if (mem is null || mem.SwapTotalBytes == 0)
        {
            return;
        }

        double used = 100.0 * (double)mem.SwapUsedBytes / mem.SwapTotalBytes;
        if (used >= 50.0)
        {
            issues.Add(new Issue
            {
                Category = Category.Memory,
                Severity = Severity.Medium,
                Title = "交换分区（Swap）使用过半",
                Description = "Swap 使用率超过 50%，物理内存可能不足，频繁换页会造成卡顿。",
                Evidence = JsonDocument.Parse($$"""{"swap_percent": {{used.ToString("F1")}}}""").RootElement,
            });
        }
    }

    private static void DiskUsage(SystemSnapshot s, List<Issue> issues)
    {
        // 只报最满的一块容量盘（Windows 后端已过滤非 Fixed 盘）
        DiskMetrics? worst = null;
        foreach (var d in s.Disks)
        {
            if (worst is null || d.UsagePercent > worst.UsagePercent)
            {
                worst = d;
            }
        }

        if (worst is null)
        {
            return;
        }

        double p = worst.UsagePercent;
        if (p >= 95.0)
        {
            issues.Add(new Issue
            {
                Category = Category.Disk,
                Severity = Severity.Critical,
                Title = "磁盘空间即将占满",
                Description = "磁盘使用率超过 95%，继续使用将导致系统异常，建议立即清理。",
                Evidence = JsonDocument.Parse($$"""{"device": "{{worst.Device}}", "percent": {{p.ToString("F1")}}}""").RootElement,
            });
        }
        else if (p >= 85.0)
        {
            issues.Add(new Issue
            {
                Category = Category.Disk,
                Severity = Severity.High,
                Title = "磁盘空间不足",
                Description = "磁盘使用率超过 85%，建议清理缓存和大文件。",
                Evidence = JsonDocument.Parse($$"""{"device": "{{worst.Device}}", "percent": {{p.ToString("F1")}}}""").RootElement,
            });
        }
    }

    private static void CpuTemperature(SystemSnapshot s, List<Issue> issues)
    {
        var cpu = s.Cpu;
        if (cpu?.TemperatureC is not { } temp)
        {
            return;
        }

        if (temp >= 95.0)
        {
            issues.Add(new Issue
            {
                Category = Category.Performance,
                Severity = Severity.Critical,
                Title = "CPU 温度过高",
                Description = "CPU 温度超过 95°C，可能降频或损坏硬件，请检查散热。",
                Evidence = JsonDocument.Parse($$"""{"temp_c": {{temp.ToString("F1")}}}""").RootElement,
            });
        }
        else if (temp >= 80.0)
        {
            issues.Add(new Issue
            {
                Category = Category.Performance,
                Severity = Severity.High,
                Title = "CPU 温度偏高",
                Description = "CPU 温度超过 80°C，建议清理风扇或降低负载。",
                Evidence = JsonDocument.Parse($$"""{"temp_c": {{temp.ToString("F1")}}}""").RootElement,
            });
        }
    }

    private static void LoadAverage(SystemSnapshot s, List<Issue> issues)
    {
        var cpu = s.Cpu;
        if (cpu?.LoadAvg1 is not { } load || load < 0)
        {
            return;
        }

        int cores = Math.Max(cpu.CoreCount, 1);
        if (load >= cores * 2.0)
        {
            issues.Add(new Issue
            {
                Category = Category.Performance,
                Severity = Severity.Medium,
                Title = "系统整体繁忙",
                Description = "1 分钟平均负载达到核心数的 2 倍以上，系统可能接近过载。",
                Evidence = JsonDocument.Parse($$"""{"load_avg_1": {{load.ToString("F1")}}, "cores": {{cores}}}""").RootElement,
            });
        }
    }
}