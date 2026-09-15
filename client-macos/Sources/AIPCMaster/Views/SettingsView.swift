// 设置：云端同步（预留）+ 关于 + 日志。

import SwiftUI
import AppKit

struct SettingsView: View {
    @EnvironmentObject var vm: MainViewModel

    var body: some View {
        Form {
            Section("云端同步（预留）") {
                Toggle("启用服务器同步", isOn: $vm.settings.isServerSyncEnabled)
                TextField("API 地址", text: $vm.settings.apiBaseURL)
                    .textFieldStyle(.roundedBorder)
                Text("关闭时所有功能在本机运行，不上传任何数据。")
                    .font(.caption)
                    .foregroundColor(.textSecondary)
            }

            Section("采样与算法") {
                LabeledContent("采样节奏", value: "正常 5 秒 / 检测到问题 0.5 秒")
                LabeledContent("健康分算法", value: "前 3 严重问题扣分，下限 20")
                LabeledContent("审计日志", value: "~/Library/Application Support/AIPCMaster/logs（JSONL）")
            }

            Section("日志") {
                Button("打开日志目录") {
                    if let url = vm.logDirectoryURL {
                        NSWorkspace.shared.open(url)
                    }
                }
            }

            Section("关于") {
                LabeledContent("版本", value: "1.0.0 (macOS 13+)")
                LabeledContent("架构", value: "SwiftUI + AIPCMasterCore（零第三方依赖）")
            }
        }
        .formStyle(.grouped)
        .navigationTitle("设置")
    }
}