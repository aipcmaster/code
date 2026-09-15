// 全链路编排（对应 Windows MainViewModel）：
// - 采样节奏：正常 5s / 检测到问题 500ms 高频（PRD §5 硬性要求）；
// - 采集 → 诊断 → 建议 每 tick 重算；执行动作写审计日志（JSONL）。

import Foundation
import Combine

@MainActor
public final class MainViewModel: ObservableObject {
    @Published public private(set) var report: DiagnosisReport?
    @Published public private(set) var advice: Advice?
    @Published public private(set) var snapshot: SystemSnapshot?
    @Published public private(set) var lastAction: ExecutionResult?
    @Published public private(set) var isExecuting = false
    @Published public var settings: LocalSettings

    public private(set) var sampleCount = 0

    private let collector = MacSystemCollector()
    private let engine = DiagnosticEngine()
    private let advisor = Advisor()
    private let executor = OptimizationExecutor()
    private let logger = AuditLogger()
    private var timer: Timer?
    private var isPolling = false

    public init(settings: LocalSettings = LocalSettings()) {
        self.settings = settings
    }

    /// 审计日志目录（供「设置 → 打开日志目录」）。
    public var logDirectoryURL: URL? { logger?.logDirectoryURL }

    // MARK: - 采样循环

    public func start() {
        guard timer == nil else { return }
        poll()
        schedule(interval: settings.pollIntervalNormal)
    }

    public func stop() {
        timer?.invalidate()
        timer = nil
    }

    private func schedule(interval: TimeInterval) {
        timer?.invalidate()
        timer = Timer.scheduledTimer(withTimeInterval: interval, repeats: false) { [weak self] _ in
            Task { @MainActor in
                self?.poll()
            }
        }
    }

    private func poll() {
        guard !isPolling else { return }
        isPolling = true
        defer { isPolling = false }

        let snap = collector.collect()
        sampleCount += 1
        snapshot = snap

        let rep = engine.diagnose(snap, sessionId: "local")
        report = rep
        advice = advisor.advise(rep)

        // 有异常 → 500ms 高频观察；否则回到 5s 常态
        let abnormal = !rep.issues.isEmpty
        schedule(interval: abnormal ? settings.pollIntervalAbnormal : settings.pollIntervalNormal)
    }

    // MARK: - 动作执行（默认只读；高危动作须经 UI 确认）

    public func execute(_ suggestion: Suggestion) async {
        guard !isExecuting else { return }
        isExecuting = true
        defer { isExecuting = false }

        logger?.record(
            actionType: suggestion.actionType.rawValue,
            status: ActionStatus.approved.rawValue,
            detail: suggestion.reason,
            confirmedByUser: suggestion.requiresConfirm
        )

        // 系统调用（如 osascript 授权框）不放主线程，避免 UI 冻结
        let result = await Task.detached(priority: .userInitiated) { [executor] in
            executor.execute(suggestion)
        }.value

        lastAction = result
        logger?.record(
            actionType: suggestion.actionType.rawValue,
            status: (result.success ? ActionStatus.executed : ActionStatus.failed).rawValue,
            detail: result.message,
            confirmedByUser: suggestion.requiresConfirm,
            restorePointCreated: result.restorePointCreated
        )

        // 云端同步（预留）：开启后尽力上报，失败不影响本地
        if settings.isServerSyncEnabled,
           let url = URL(string: settings.apiBaseURL) {
            let api = ApiClient(baseURL: url)
            let status = result.success ? ActionStatus.executed.rawValue : ActionStatus.failed.rawValue
            let detail = "\(suggestion.actionType.rawValue): \(result.message)"
            _ = await api.reportAction(actionType: suggestion.actionType.rawValue, status: status, detail: detail)
        }

        // 执行后立即刷新一次
        poll()
    }
}