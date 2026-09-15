// 本地设置（UserDefaults 持久化）。

import Foundation
import Combine

public final class LocalSettings: ObservableObject {
    private enum Key {
        static let apiBaseURL = "apiBaseURL"
        static let isServerSyncEnabled = "isServerSyncEnabled"
    }

    @Published public var apiBaseURL: String {
        didSet { UserDefaults.standard.set(apiBaseURL, forKey: Key.apiBaseURL) }
    }

    @Published public var isServerSyncEnabled: Bool {
        didSet { UserDefaults.standard.set(isServerSyncEnabled, forKey: Key.isServerSyncEnabled) }
    }

    /// 采样节奏（PRD §5：正常 5s / 异常 500ms）。
    public let pollIntervalNormal: TimeInterval = 5.0
    public let pollIntervalAbnormal: TimeInterval = 0.5

    /// 默认 API 地址（server 后端；如未部署可保持关闭）。
    public static let defaultAPIBaseURL = "https://api.aipcmaster.com"

    public init() {
        let d = UserDefaults.standard
        apiBaseURL = d.string(forKey: Key.apiBaseURL) ?? Self.defaultAPIBaseURL
        isServerSyncEnabled = d.bool(forKey: Key.isServerSyncEnabled)
    }
}