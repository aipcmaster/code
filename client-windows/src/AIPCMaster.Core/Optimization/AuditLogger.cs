// 本地审计日志 —— 对齐 ERD §3.12 optimization_logs 与 SD §2.3「完整审计日志」。
// JSON Lines 格式（每行一个对象），零依赖，追加写。

using System.Text.Json;

namespace AIPCMaster.Core.Optimization;

/// 一条本地审计日志。
public sealed class AuditLogEntry
{
    public string Id { get; set; } = Guid.NewGuid().ToString("N");
    public long TimestampUnixMs { get; set; } = DateTimeOffset.UtcNow.ToUnixTimeMilliseconds();

    /// 动作类型（snake_case）。
    public string ActionType { get; set; } = "";

    /// 状态（suggested/approved/executed/rolled_back/failed）。
    public string Status { get; set; } = "";

    /// 触发来源：user（用户确认执行）/ system（自动）。
    public string Trigger { get; set; } = "user";

    public string Detail { get; set; } = "";
    public bool RestorePointCreated { get; set; }

    /// 是否用户显式确认（高危动作必需）。
    public bool ConfirmedByUser { get; set; }
}

/// 本地审计日志写入器（追加 JSONL）。
public sealed class AuditLogger : IDisposable
{
    private readonly StreamWriter _writer;
    private readonly object _lock = new();

    /// 日志目录：默认 %LOCALAPPDATA%/AIPCMaster/logs。
    public AuditLogger(string? logDir = null)
    {
        string dir = logDir
            ?? Path.Combine(
                Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData),
                "AIPCMaster", "logs");

        Directory.CreateDirectory(dir);
        string file = Path.Combine(dir, $"audit-{DateTime.UtcNow:yyyyMMdd}.jsonl");
        _writer = new StreamWriter(file, append: true) { AutoFlush = true };
    }

    public void Append(AuditLogEntry entry)
    {
        lock (_lock)
        {
            _writer.WriteLine(JsonSerializer.Serialize(entry));
        }
    }

    /// 便捷方法：记录一次动作状态变化。
    public void Record(string actionType, string status, string detail,
        bool confirmedByUser = false, bool restorePointCreated = false)
    {
        Append(new AuditLogEntry
        {
            ActionType = actionType,
            Status = status,
            Detail = detail,
            ConfirmedByUser = confirmedByUser,
            RestorePointCreated = restorePointCreated,
        });
    }

    public void Dispose() => _writer.Dispose();
}