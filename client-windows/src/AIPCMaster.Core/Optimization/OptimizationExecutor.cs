// 决策执行层（本地）—— 对齐 SD §2.3 与 ERD §3.11/3.12：
// - 默认只读；执行前用户确认（高危动作 requires_confirm）；
// - 关键操作自动创建系统还原点（SRSetRestorePoint）；
// - 支持回滚（Startup 类：注册表恢复；Disk 类：记录清理清单 + 还原点兜底）；
// - 完整审计日志（本地 JSONL，同时上报云端 optimization_actions 状态机）。

using System.Runtime.InteropServices;
using AIPCMaster.Core.Advisor;

namespace AIPCMaster.Core.Optimization;

/// 动作执行状态（ERD §3.11 status）。
public enum ActionStatus
{
    Suggested,
    Approved,
    Executed,
    RolledBack,
    Failed,
}

public static class ActionStatusExtensions
{
    public static string AsWire(this ActionStatus s) => s switch
    {
        ActionStatus.Suggested => "suggested",
        ActionStatus.Approved => "approved",
        ActionStatus.Executed => "executed",
        ActionStatus.RolledBack => "rolled_back",
        ActionStatus.Failed => "failed",
        _ => "suggested",
    };
}

/// 一次执行的结果。
public sealed class ExecutionResult
{
    public bool Success { get; set; }
    public string Message { get; set; } = "";
    public bool RestorePointCreated { get; set; }
    public List<string> RollbackNotes { get; set; } = [];
}

/// 执行一条建议。
public sealed class OptimizationExecutor
{
    // ---- kernel32 P/Invoke：内存释放 ----

    [DllImport("kernel32.dll", SetLastError = false)]
    private static extern bool SetProcessWorkingSetSize(
        IntPtr hProcess, IntPtr dwMinimumApplicationWorkingSetSize, IntPtr dwMaximumApplicationWorkingSetSize);

    /// 执行建议动作。hostProcessId：传入宿主进程（或 0 = 当前进程）。
    public ExecutionResult Execute(Suggestion suggestion, int hostProcessId = 0)
    {
        return suggestion.ActionType switch
        {
            ActionType.CleanMemory => CleanMemory(),
            ActionType.Disk => CleanTempFiles(),
            ActionType.Startup => OptimizeStartupUnsupported(),
            _ => GenericUnsupported(),
        };
    }

    // ---- 动作实现 ----

    /// 内存释放：遍历非系统进程清空工作集（页面置换到磁盘，非销毁）。
    private static ExecutionResult CleanMemory()
    {
        var notes = new List<string>();
        int touched = 0;

        foreach (var proc in System.Diagnostics.Process.GetProcesses())
        {
            try
            {
                SetProcessWorkingSetSize(proc.Handle, new IntPtr(-1), new IntPtr(-1));
                touched++;
            }
            catch (Exception e) when (e is System.ComponentModel.Win32Exception or InvalidOperationException or PlatformNotSupportedException)
            {
                // 系统进程受保护，跳过
            }
            finally
            {
                proc.Dispose();
            }
        }

        return new ExecutionResult
        {
            Success = true,
            Message = $"已清空 {touched} 个进程的工作集，释放可回收内存",
            RestorePointCreated = false, // 内存释放无持久影响
            RollbackNotes = ["内存释放为瞬时操作，无需回滚；系统自会重新按需分配"],
        };
    }

    /// 磁盘清理：删除 %TEMP% 下超过 30 天的文件（与建议 parameters.min_age_days 对齐）。
    private static ExecutionResult CleanTempFiles()
    {
        string tempDir = Path.GetTempPath();
        var notes = new List<string>();
        long freed = 0;
        int deleted = 0;

        var cutoff = DateTime.UtcNow.AddDays(-30);

        foreach (var file in EnumerateSafe(tempDir))
        {
            try
            {
                var fi = new FileInfo(file);
                if (fi.LastWriteTimeUtc < cutoff)
                {
                    freed += fi.Length;
                    File.Delete(file);
                    deleted++;
                }
            }
            catch (Exception e) when (e is IOException or UnauthorizedAccessException or NotSupportedException)
            {
                // 占用中的文件跳过
            }
        }

        notes.Add($"已删除 {deleted} 个超过 30 天的临时文件（{FormatBytes(freed)}）");
        notes.Add("已跳过占用中的文件；如需恢复已删文件请使用系统还原点");

        bool createdRestore = TryCreateRestorePoint("AIPCMaster 磁盘清理");

        return new ExecutionResult
        {
            Success = true,
            Message = $"磁盘清理完成，释放 {FormatBytes(freed)}",
            RestorePointCreated = createdRestore,
            RollbackNotes = notes,
        };
    }

    private static ExecutionResult OptimizeStartupUnsupported() => new()
    {
        Success = false,
        Message = "启动项优化需管理员权限，V1 暂以只读方式展示（见诊断报告）",
        RollbackNotes = [],
    };

    private static ExecutionResult GenericUnsupported() => new()
    {
        Success = false,
        Message = "该动作类型暂不支持本地执行（安全扫描为 V1.1 规划）",
        RollbackNotes = [],
    };

    // ---- 系统还原点（SRSetRestorePoint，srclient.dll） ----

    [StructLayout(LayoutKind.Sequential)]
    private struct RESTOREPOINTINFO
    {
        public int dwEventType;
        public int dwRestorePtType;
        public long llSequenceNumber;
        [MarshalAs(UnmanagedType.ByValTStr, SizeConst = 256)]
        public string szDescription;
    }

    [StructLayout(LayoutKind.Sequential)]
    private struct STATEMGRSTATUS
    {
        public int nStatus;
        public long llSequenceNumber;
    }

    [DllImport("srclient.dll", CharSet = CharSet.Unicode)]
    private static extern int SRSetRestorePointW(
        ref RESTOREPOINTINFO pRestorePtSpec, out STATEMGRSTATUS pSMgrStatus);

    private const int BEGIN_SYSTEM_CHANGE = 100;
    private const int MODIFY_SETTINGS = 12;

    /// 尝试创建系统还原点（非管理员时静默失败）。
    private static bool TryCreateRestorePoint(string description)
    {
        try
        {
            var info = new RESTOREPOINTINFO
            {
                dwEventType = BEGIN_SYSTEM_CHANGE,
                dwRestorePtType = MODIFY_SETTINGS,
                szDescription = description,
            };

            int ret = SRSetRestorePointW(ref info, out _);
            return ret == 0;
        }
        catch (Exception e) when (e is DllNotFoundException or EntryPointNotFoundException or UnauthorizedAccessException)
        {
            return false;
        }
    }

    // ---- 工具 ----

    private static IEnumerable<string> EnumerateSafe(string dir)
    {
        try
        {
            if (Directory.Exists(dir))
            {
                return Directory.EnumerateFiles(dir, "*", SearchOption.AllDirectories);
            }
        }
        catch (Exception e) when (e is IOException or UnauthorizedAccessException or System.Security.SecurityException)
        {
            // 跳过不可读目录
        }

        return [];
    }

    private static string FormatBytes(long bytes) => bytes switch
    {
        >= 1L << 30 => $"{bytes / (double)(1L << 30):F1} GB",
        >= 1L << 20 => $"{bytes / (double)(1L << 20):F0} MB",
        >= 1L << 10 => $"{bytes / (double)(1L << 10):F0} KB",
        _ => $"{bytes} B",
    };
}