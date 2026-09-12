// AI 推理层（规则化引用实现）—— 对齐 ERD §3.11 optimization_actions
// 与 core-rust crates/aipcmaster-advisor/src/lib.rs。
//
// 输入诊断报告 → 输出根因摘要 + 优化动作建议（action_type / risk_level /
// requires_confirm / parameters）。高危动作必须用户确认（SD §2.3 默认只读）。

using System.Text.Json;
using System.Text.Json.Serialization;
using AIPCMaster.Core.Diagnosis;

namespace AIPCMaster.Core.Advisor;

/// 优化动作类型（ERD §3.11 action_type，snake_case）。
[JsonConverter(typeof(JsonStringEnumConverter<ActionType>))]
public enum ActionType
{
    [JsonPropertyName("clean_memory")]
    CleanMemory,

    [JsonPropertyName("startup")]
    Startup,

    [JsonPropertyName("disk")]
    Disk,

    [JsonPropertyName("generic")]
    Generic,
}

/// 风险级别（ERD §3.11 risk_level）。
[JsonConverter(typeof(JsonStringEnumConverter<RiskLevel>))]
public enum RiskLevel
{
    [JsonPropertyName("low")]
    Low,

    [JsonPropertyName("medium")]
    Medium,

    [JsonPropertyName("high")]
    High,
}

public static class RiskLevelExtensions
{
    public static string Display(this RiskLevel r) => r switch
    {
        RiskLevel.Low => "低风险",
        RiskLevel.Medium => "中风险",
        RiskLevel.High => "高风险",
        _ => "未知",
    };
}

/// 一条优化动作建议（ERD §3.11 字段映射）。
public sealed class Suggestion
{
    public ActionType ActionType { get; set; }
    public RiskLevel RiskLevel { get; set; }

    /// 高危动作必须用户确认（SD §2.3 默认只读）。
    public bool RequiresConfirm { get; set; }

    public JsonElement Parameters { get; set; }
    public string Reason { get; set; } = "";
    public Category Category { get; set; }
}

/// 一次推理的结果：根因摘要 + 动作建议列表。
public sealed class Advice
{
    public string RootCauseSummary { get; set; } = "";

    /// 按风险升序排列的动作建议（低风险优先执行）。
    public List<Suggestion> Suggestions { get; set; } = [];

    public bool IsEmpty => Suggestions.Count == 0;
}

/// AI 推理引擎（规则化引用实现，纯计算可测）。
public sealed class Advisor
{
    public Advice Advise(DiagnosisReport report)
    {
        var suggestions = report.Issues
            .Where(i => i.Severity >= Severity.Medium)
            .Select(SuggestFor)
            .ToList();

        suggestions = Dedup(suggestions);

        // 风险升序（低风险优先执行）
        suggestions = suggestions
            .OrderBy(s => s.RiskLevel)
            .ToList();

        return new Advice
        {
            RootCauseSummary = BuildSummary(report, suggestions),
            Suggestions = suggestions,
        };
    }

    // ---- 建议生成（与 core-rust advisor::suggest_for 对齐） ----

    private static Suggestion SuggestFor(Issue issue)
    {
        switch (issue.Category)
        {
            case Category.Memory:
                bool memCritical = issue.Severity == Severity.Critical;
                return new Suggestion
                {
                    ActionType = ActionType.CleanMemory,
                    RiskLevel = memCritical ? RiskLevel.Medium : RiskLevel.Low,
                    RequiresConfirm = memCritical,
                    Parameters = JsonDocument.Parse("""{"target": "page_cache", "amount": "auto"}""").RootElement,
                    Reason = memCritical
                        ? "内存严重偏高，建议清理缓存页面释放内存"
                        : "内存使用率偏高，建议清理缓存页面释放内存",
                    Category = issue.Category,
                };

            case Category.Disk:
                bool diskHigh = issue.Severity >= Severity.High;
                return new Suggestion
                {
                    ActionType = ActionType.Disk,
                    RiskLevel = diskHigh ? RiskLevel.High : RiskLevel.Medium,
                    RequiresConfirm = diskHigh,
                    Parameters = JsonDocument.Parse("""{"target": "temp_files", "min_age_days": 30}""").RootElement,
                    Reason = "磁盘空间紧张，建议清理临时文件与缓存",
                    Category = issue.Category,
                };

            case Category.Performance:
                return new Suggestion
                {
                    ActionType = ActionType.Startup,
                    RiskLevel = RiskLevel.Low,
                    RequiresConfirm = false,
                    Parameters = JsonDocument.Parse("""{"target": "startup_items"}""").RootElement,
                    Reason = "CPU/负载偏高，建议检查开机自启项与后台进程",
                    Category = issue.Category,
                };

            default: // Security
                return new Suggestion
                {
                    ActionType = ActionType.Generic,
                    RiskLevel = RiskLevel.High,
                    RequiresConfirm = true,
                    Parameters = JsonDocument.Parse("""{"target": "security_scan"}""").RootElement,
                    Reason = "存在安全风险，建议执行安全扫描并更新系统",
                    Category = issue.Category,
                };
        }
    }

    /// 同类动作去重：保留风险最高的（reason 合并）。
    private static List<Suggestion> Dedup(List<Suggestion> items)
    {
        var map = new Dictionary<(ActionType, Category), Suggestion>();
        foreach (var item in items)
        {
            if (map.TryGetValue((item.ActionType, item.Category), out var existing))
            {
                if (item.RiskLevel > existing.RiskLevel)
                {
                    existing.RiskLevel = item.RiskLevel;
                    existing.RequiresConfirm = item.RequiresConfirm;
                }

                existing.Reason = $"{existing.Reason}；{item.Reason}";
            }
            else
            {
                map[(item.ActionType, item.Category)] = item;
            }
        }

        return map.Values.ToList();
    }

    // ---- 根因摘要（与 core-rust build_summary 对齐） ----

    private static string BuildSummary(DiagnosisReport report, List<Suggestion> suggestions)
    {
        if (suggestions.Count == 0)
        {
            return $"系统健康分 {report.HealthScore}，未发现需要优化的问题。";
        }

        var worst = report.Issues.Max(i => i.Severity);
        string worstWord = worst switch
        {
            Severity.Critical => "严重",
            Severity.High => "较高",
            Severity.Medium => "中等",
            _ => "轻微",
        };

        var kinds = suggestions
            .Select(s => s.ActionType.ToString().ToLowerInvariant())
            .Distinct()
            .OrderBy(x => x)
            .ToList();

        var actionNames = suggestions
            .Select(s => s.ActionType.ToString().ToLowerInvariant())
            .ToList();

        return $"健康分 {report.HealthScore}。存在{worstWord}风险问题（{string.Join("、", kinds)}），建议执行 {suggestions.Count} 项优化：{string.Join("、", actionNames)}。";
    }
}