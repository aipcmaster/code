// 云端对接（预留）—— 对应 Windows ApiClient，对接 server（axum REST API）。
// 默认关闭；开启后为尽力而为（best-effort），失败不影响本地功能。

import Foundation

public final class ApiClient {
    public let baseURL: URL

    public init(baseURL: URL) {
        self.baseURL = baseURL
    }

    /// 上报一次优化动作状态（ERD §3.11 状态机 suggested→executed）。
    public func reportAction(actionType: String, status: String, detail: String) async -> Bool {
        var request = URLRequest(url: baseURL.appendingPathComponent("api/v1/optimizations/actions"))
        request.httpMethod = "POST"
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")

        let body: [String: Any] = [
            "action_type": actionType,
            "status": status,
            "detail": detail,
            "device_id": DeviceIdentity.shared.id,
        ]
        request.httpBody = try? JSONSerialization.data(withJSONObject: body)

        do {
            let (_, response) = try await URLSession.shared.data(for: request)
            return (response as? HTTPURLResponse)?.statusCode ?? 0 < 500
        } catch {
            return false
        }
    }

    /// 首次启动生成持久设备 ID（PRD §4.4），macOS 用 UUID 写入 UserDefaults。
    public enum DeviceIdentity {
        public static let shared = DeviceIdentity()
        public let id: String

        private init() {
            let key = "aipcmaster_device_id"
            if let existing = UserDefaults.standard.string(forKey: key) {
                id = existing
            } else {
                let fresh = UUID().uuidString.lowercased()
                UserDefaults.standard.set(fresh, forKey: key)
                id = fresh
            }
        }
    }
}