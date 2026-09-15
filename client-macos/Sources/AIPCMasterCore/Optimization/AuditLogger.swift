// 本地审计日志 —— 对齐 ERD §3.12 optimization_logs 与 SD §2.3「完整审计日志」。
// JSON Lines 格式（每行一个对象），零依赖，追加写。
// macOS 日志目录：~/Library/Application Support/AIPCMaster/logs/（对应 Windows %LOCALAPPDATA%）。

import Foundation

/// 一条本地审计日志。
public struct AuditLogEntry {
    public var id: String = UUID().uuidString.lowercased()
    public var timestampUnixMs: Int64 = Int64(Date().timeIntervalSince1970 * 1000)

    /// 动作类型（snake_case）。
    public var actionType: String = ""

    /// 状态（suggested / approved / executed / rolled_back / failed）。
    public var status: String = ""

    /// 触发来源：user（用户确认执行）/ system（自动）。
    public var trigger: String = "user"

    public var detail: String = ""
    public var restorePointCreated: Bool = false

    /// 是否用户显式确认（高危动作必需）。
    public var confirmedByUser: Bool = false

    public init() {}
}

/// 本地审计日志写入器（追加 JSONL）。
public final class AuditLogger {
    private let fileHandle: FileHandle
    private let lock = NSLock()

    /// 日志目录：默认 ~/Library/Application Support/AIPCMaster/logs。
    public init?(logDir: String? = nil) {
        let base: URL
        if let logDir {
            base = URL(fileURLWithPath: logDir)
        } else {
            guard let appSupport = FileManager.default.urls(
                for: .applicationSupportDirectory,
                in: .userDomainMask
            ).first else { return nil }
            base = appSupport
        }

        let dir = base.appendingPathComponent("AIPCMaster/logs", isDirectory: true)
        do {
            try FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
        } catch {
            return nil
        }

        let df = DateFormatter()
        df.dateFormat = "yyyyMMdd"
        let file = dir.appendingPathComponent("audit-\(df.string(from: Date())).jsonl")

        if !FileManager.default.fileExists(atPath: file.path) {
            FileManager.default.createFile(atPath: file.path, contents: nil)
        }

        guard let fh = try? FileHandle(forWritingTo: file) else { return nil }
        fh.seekToEndOfFile()
        self.fileHandle = fh
    }

    /// 追加一条日志条目。
    public func append(_ entry: AuditLogEntry) {
        lock.lock()
        defer { lock.unlock() }

        let dict: [String: Any] = [
            "id": entry.id,
            "timestamp_unix_ms": entry.timestampUnixMs,
            "action_type": entry.actionType,
            "status": entry.status,
            "trigger": entry.trigger,
            "detail": entry.detail,
            "restore_point_created": entry.restorePointCreated,
            "confirmed_by_user": entry.confirmedByUser,
        ]

        guard let data = try? JSONSerialization.data(withJSONObject: dict),
              var line = String(data: data, encoding: .utf8) else {
            return
        }
        line += "\n"
        if let lineData = line.data(using: .utf8) {
            fileHandle.write(lineData)
        }
    }

    /// 便捷方法：记录一次动作状态变化。
    public func record(
        actionType: String,
        status: String,
        detail: String,
        confirmedByUser: Bool = false,
        restorePointCreated: Bool = false
    ) {
        var entry = AuditLogEntry()
        entry.actionType = actionType
        entry.status = status
        entry.detail = detail
        entry.confirmedByUser = confirmedByUser
        entry.restorePointCreated = restorePointCreated
        append(entry)
    }

    /// 获取日志目录路径（供 UI "打开日志" 按钮使用）。
    public var logDirectoryURL: URL? {
        (try? FileManager.default.url(
            for: .applicationSupportDirectory,
            in: .userDomainMask,
            appropriateFor: nil,
            create: false
        ))?.appendingPathComponent("AIPCMaster/logs", isDirectory: true)
    }

    deinit {
        try? fileHandle.close()
    }
}