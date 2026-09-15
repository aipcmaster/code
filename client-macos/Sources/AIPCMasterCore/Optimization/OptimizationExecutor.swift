// 决策执行层（macOS 本地）—— 对齐 SD §2.3 与 ERD §3.11/3.12。
//
// macOS 平台动作（与 client-windows OptimizationExecutor.cs 对比）：
// - CleanMemory：执行 /usr/bin/purge（释放可回收内存页缓存），
//   需管理员权限 → 通过 osascript 弹出系统授权框；
// - Disk：清理 ~/Library/Caches 中超 30 天的文件，
//   移入废纸篓（可恢复），比 Windows File.Delete 更安全；
// - Startup：V1 只读（列表 LaunchAgents 目录），执行返回 unsupported；
// - Generic/Security：V1 占位，返回 unsupported。
//
// macOS 无 System Restore → 废纸篓兜底（SS-01 推理策略 V1 降级方案）。
// 完整审计日志（本地 JSONL，对齐 ERD §3.12）。

import Foundation

/// 动作执行状态（ERD §3.11 status）。
public enum ActionStatus: String, Sendable {
    case suggested
    case approved
    case executed
    case rolledBack = "rolled_back"
    case failed
}

/// 一次执行的结果。
public struct ExecutionResult: Sendable {
    public var success: Bool
    public var message: String
    public var restorePointCreated: Bool
    public var rollbackNotes: [String]

    public init(success: Bool = false, message: String = "", restorePointCreated: Bool = false, rollbackNotes: [String] = []) {
        self.success = success
        self.message = message
        self.restorePointCreated = restorePointCreated
        self.rollbackNotes = rollbackNotes
    }
}

/// macOS 优化动作执行器。
public final class OptimizationExecutor {
    public init() {}

    public func execute(_ suggestion: Suggestion) -> ExecutionResult {
        switch suggestion.actionType {
        case .cleanMemory:
            return cleanMemory()
        case .disk:
            return cleanCaches(minAgeDays: suggestion.parameters["min_age_days"] as? Int ?? 30)
        case .startup:
            return startupUnsupported()
        case .generic:
            return genericUnsupported()
        }
    }

    // MARK: - 内存释放（/usr/bin/purge，需管理员权限）

    private func cleanMemory() -> ExecutionResult {
        let ok = runPrivileged("/usr/bin/purge")
        if ok {
            return ExecutionResult(
                success: true,
                message: "已执行 purge，释放可回收的内存",
                restorePointCreated: false,
                rollbackNotes: ["purge 为瞬时操作，系统自会重新按需分配，无需回滚"]
            )
        } else {
            return ExecutionResult(
                success: false,
                message: "清理内存需要管理员权限（授权框已弹出或被拒绝）",
                rollbackNotes: []
            )
        }
    }

    // MARK: - 磁盘清理（~/Library/Caches 超龄文件 → 废纸篓，可恢复）

    private func cleanCaches(minAgeDays: Int = 30) -> ExecutionResult {
        let fm = FileManager.default
        guard let cacheDir = fm.urls(for: .cachesDirectory, in: .userDomainMask).first else {
            return ExecutionResult(success: false, message: "无法访问缓存目录")
        }

        let cutoff = Date().addingTimeInterval(-Double(minAgeDays) * 86400)
        var filesToTrash: [URL] = []
        var totalBytes: Int64 = 0

        guard let enumerator = fm.enumerator(
            at: cacheDir,
            includingPropertiesForKeys: [.isRegularFileKey, .contentModificationDateKey, .fileSizeKey],
            options: [.skipsHiddenFiles, .skipsPackageDescendants]
        ) else {
            return ExecutionResult(success: false, message: "无法枚举缓存目录")
        }

        // 缓存目录结构：~/Library/Caches/<BundleID>/文件。
        // 跳过系统管理的目录（com.apple.*），避免运行时应用崩溃。
        let topLevelComponents = cacheDir.pathComponents.count

        for case let url as URL in enumerator {
            let components = url.pathComponents.count

            // 一级子目录若是 com.apple.* → 跳过整棵子树
            if components == topLevelComponents + 1,
               (url.lastPathComponent).hasPrefix("com.apple.") {
                enumerator.skipDescendants()
                continue
            }

            guard let values = try? url.resourceValues(forKeys: [.isRegularFileKey, .contentModificationDateKey, .fileSizeKey]),
                  values.isRegularFile == true,
                  let mod = values.contentModificationDate, mod < cutoff else {
                continue
            }

            filesToTrash.append(url)
            totalBytes += Int64(values.fileSize ?? 0)

            if filesToTrash.count >= 5000 { break } // 防止单次清理过量
        }

        var trashedCount = 0
        for url in filesToTrash {
            do {
                try fm.trashItem(at: url, resultingItemURL: nil)
                trashedCount += 1
            } catch {
                // 占用中 / 权限不足：跳过，不中断流程
            }
        }

        return ExecutionResult(
            success: true,
            message: "已移入废纸篓 \(trashedCount) 个超过 \(minAgeDays) 天的缓存文件（约 \(formatBytes(totalBytes))，可自废纸篓恢复）",
            restorePointCreated: false,
            rollbackNotes: [
                "文件已移入废纸篓而非直接删除，如需恢复请在 30 天内访问废纸篓",
                "macOS 无系统还原点；V1 以废纸篓兜底（比 Windows File.Delete 更安全）",
                "如需额外保障，请确保 Time Machine 已开启"
            ]
        )
    }

    // MARK: - V1 占位

    private func startupUnsupported() -> ExecutionResult {
        ExecutionResult(
            success: false,
            message: "启动项优化需系统权限，V1 暂以只读方式展示（见诊断报告）",
            rollbackNotes: []
        )
    }

    private func genericUnsupported() -> ExecutionResult {
        ExecutionResult(
            success: false,
            message: "该动作类型暂不支持本地执行（安全扫描为 V1.1 规划）",
            rollbackNotes: []
        )
    }

    // MARK: - macOS 管理员权限执行（osascript 弹出授权框）

    private func runPrivileged(_ command: String) -> Bool {
        let escaped = command.replacingOccurrences(of: "\"", with: "\\\"")
        let script = "do shell script \"\(escaped)\" with administrator privileges"
        let p = Process()
        p.executableURL = URL(fileURLWithPath: "/usr/bin/osascript")
        p.arguments = ["-e", script]
        p.standardOutput = FileHandle.nullDevice
        p.standardError = FileHandle.nullDevice
        do {
            try p.run()
            p.waitUntilExit()
            return p.terminationStatus == 0
        } catch {
            return false
        }
    }

    // MARK: - 工具

    private func formatBytes(_ bytes: Int64) -> String {
        switch bytes {
        case let b where b >= (1 << 30): return String(format: "%.1f GB", Double(b) / Double(1 << 30))
        case let b where b >= (1 << 20): return "\(b / (1 << 20)) MB"
        case let b where b >= (1 << 10): return "\(b / (1 << 10)) KB"
        default: return "\(bytes) B"
        }
    }
}