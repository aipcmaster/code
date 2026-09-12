// 诊断领域模型 —— 对齐 ERD §3.10 issues 表 + core-rust diagnose crate。

using System.Text.Json.Serialization;

namespace AIPCMaster.Core.Diagnosis;

/// 问题类别（ERD §3.10 category，snake_case 序列化与 Rust 一致）。
[JsonConverter(typeof(JsonStringEnumConverter<Category>))]
public enum Category
{
    [JsonPropertyName("performance")]
    Performance,

    [JsonPropertyName("disk")]
    Disk,

    [JsonPropertyName("memory")]
    Memory,

    [JsonPropertyName("security")]
    Security,
}

/// 严重度。Critical > High > Medium > Low。
[JsonConverter(typeof(JsonStringEnumConverter<Severity>))]
public enum Severity
{
    [JsonPropertyName("low")]
    Low,

    [JsonPropertyName("medium")]
    Medium,

    [JsonPropertyName("high")]
    High,

    [JsonPropertyName("critical")]
    Critical,
}

public static class SeverityExtensions
{
    /// 严重度对应的健康分扣分（与 core-rust health.rs 一致）。
    public static int HealthPenalty(this Severity s) => s switch
    {
        Severity.Critical => 40,
        Severity.High => 25,
        Severity.Medium => 15,
        Severity.Low => 5,
        _ => 0,
    };

    public static string Display(this Severity s) => s switch
    {
        Severity.Critical => "严重",
        Severity.High => "较高",
        Severity.Medium => "中等",
        Severity.Low => "轻微",
        _ => "未知",
    };

    public static string Display(this Category c) => c switch
    {
        Category.Performance => "性能",
        Category.Disk => "磁盘",
        Category.Memory => "内存",
        Category.Security => "安全",
        _ => "未知",
    };
}

/// 检测到的一个问题（ERD §3.10 字段映射）。
public sealed class Issue
{
    public Category Category { get; set; }
    public Severity Severity { get; set; }
    public string Title { get; set; } = "";
    public string Description { get; set; } = "";

    /// 证据（关键指标值），JSON 结构，适配 ERD evidence_json。
    public System.Text.Json.JsonElement Evidence { get; set; }
}

/// 一次诊断的结果（与 Rust DiagnosisReport 对齐）。
public sealed class DiagnosisReport
{
    /// 健康分 0~100（100 为最佳）。
    public int HealthScore { get; set; }

    /// 问题列表（按严重度降序）。
    public List<Issue> Issues { get; set; } = [];

    /// 诊断完成时间（Unix 毫秒）。
    public long GeneratedAtUnixMs { get; set; }

    /// 会话标识（由上层生成）。
    public string SessionId { get; set; } = "";
}