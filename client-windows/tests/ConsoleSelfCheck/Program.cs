// AIPCMaster Windows 自检程序（无需测试框架）：
// 验证 采集 → 诊断 → 建议 → 健康分算法 全链路，并输出报告。
// 用法：dotnet run --project tests/ConsoleSelfCheck （在 Windows 上执行）

using AIPCMaster.Core.Advisor;
using AIPCMaster.Core.Collectors;
using AIPCMaster.Core.Diagnosis;

int passed = 0;
int failed = 0;

void Check(string name, bool ok, string? detail = null)
{
    if (ok)
    {
        passed++;
        Console.WriteLine($"  ✅ {name}");
    }
    else
    {
        failed++;
        Console.WriteLine($"  ❌ {name} {(detail is null ? "" : $"—— {detail}")}");
    }
}

Console.WriteLine("════════════════════════════════════════");
Console.WriteLine(" AIPCMaster 自检 · 采集→诊断→建议 全链路");
Console.WriteLine($" 时间: {DateTime.Now:yyyy-MM-dd HH:mm:ss}");
Console.WriteLine("════════════════════════════════════════");

// 1. 采集
Console.WriteLine("\n[1/4] 采集");
var collector = new WindowsSystemCollector();
var snap = collector.CollectFull();

Check("采集返回快照", snap is not null);
Check("CPU 指标存在", snap.Cpu is not null, $"UsagePercent={snap.Cpu?.UsagePercent:F1}%");
Check("CPU 使用率在 0-100", snap.Cpu is { UsagePercent: >= 0 and <= 100 });
Check("核心数 > 0", (snap.Cpu?.CoreCount ?? 0) > 0);
Check("内存指标存在", snap.Memory is not null);
Check("内存总量 > 0", (snap.Memory?.TotalBytes ?? 0) > 0);
Check("内存使用率 0-100", snap.Memory is { UsagePercent: >= 0 and <= 100 });
Console.WriteLine($"     CPU {snap.Cpu?.UsagePercent:F1}% | 内存 {snap.Memory?.UsagePercent:F1}% | 盘 {snap.Disks.Count} 个");

// 2. 诊断
Console.WriteLine("\n[2/4] 诊断（规则移植阈值对齐 core-rust）");
var engine = new DiagnosticEngine();
var report = engine.Diagnose(snap, "selfcheck");
Check("诊断报告生成", report is not null);
Check("健康分 20-100", report.HealthScore is >= 20 and <= 100, $"score={report.HealthScore}");
Console.WriteLine($"     健康分 {report.HealthScore}，问题 {report.Issues.Count} 项");
foreach (var issue in report.Issues)
{
    Console.WriteLine($"     · [{issue.Severity}] {issue.Title}");
}

// 3. 建议
Console.WriteLine("\n[3/4] 优化建议（ERD 3.11）");
var advisor = new Advisor();
var advice = advisor.Advise(report);
Check("建议生成", advice is not null);
Check("根因摘要非空", !string.IsNullOrEmpty(advice.RootCauseSummary));
Check("建议按风险升序", IsSortedByRisk(advice.Suggestions));
Console.WriteLine($"     {advice.RootCauseSummary}");

// 4. 健康分算法单元校验（移植 core-rust health.rs 的已知用例）
Console.WriteLine("\n[4/4] 健康分算法（移植对齐）");
Check("空问题=100", HealthScore.Compute([]) == 100);
var critical = new Issue { Category = Category.Performance, Severity = Severity.Critical, Title = "t", Description = "d", Evidence = default };
Check("单Critical=60", HealthScore.Compute([critical]) == 60);
var high = new Issue { Category = Category.Performance, Severity = Severity.High, Title = "t", Description = "d", Evidence = default };
var medium = new Issue { Category = Category.Performance, Severity = Severity.Medium, Title = "t", Description = "d", Evidence = default };
Check("High+Medium=60", HealthScore.Compute([high, medium]) == 60);
var lows = Enumerable.Range(0, 4)
    .Select(_ => new Issue { Category = Category.Performance, Severity = Severity.Low, Title = "t", Description = "d" })
    .ToList();
Check("前3Low取扣=85", HealthScore.Compute(lows) == 85);
var floors = Enumerable.Repeat(critical, 4).ToList();
Check("封底=20", HealthScore.Compute(floors) == 20);

// 汇总
Console.WriteLine("\n════════════════════════════════════════");
Console.WriteLine($" 结果: {passed} 通过, {failed} 失败");
Console.WriteLine("════════════════════════════════════════");
return failed == 0 ? 0 : 1;

static bool IsSortedByRisk(List<Suggestion> items)
{
    for (int i = 1; i < items.Count; i++)
    {
        if (items[i].RiskLevel < items[i - 1].RiskLevel)
        {
            return false;
        }
    }

    return true;
}