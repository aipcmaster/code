// 视觉令牌 —— 与官网设计调色板对齐（text-faint #7D879B 等，满足明显对比度 ≥4.5）。

import SwiftUI

extension Color {
    static let appBackground = Color(hex: 0xF8FAFC)
    static let appCard = Color(hex: 0xFFFFFF)
    static let textPrimary = Color(hex: 0x111827)
    static let textSecondary = Color(hex: 0x4B5563)
    static let textFaint = Color(hex: 0x7D879B)
    static let accent = Color(hex: 0x2563EB)
    static let success = Color(hex: 0x16A34A)
    static let warning = Color(hex: 0xD97706)
    static let danger = Color(hex: 0xDC2626)

    // 严重度色（语义化）
    static let severityCritical = Color(hex: 0xDC2626)
    static let severityHigh = Color(hex: 0xEA580C)
    static let severityMedium = Color(hex: 0xD97706)
    static let severityLow = Color(hex: 0x16A34A)

    /// 由 0xRRGGBB 初始化。
    init(hex: UInt32) {
        self.init(
            red: Double((hex >> 16) & 0xFF) / 255,
            green: Double((hex >> 8) & 0xFF) / 255,
            blue: Double(hex & 0xFF) / 255
        )
    }
}