// 优化中心：建议列表 + 执行（高危动作弹确认框，SD §2.3 默认只读）。

import SwiftUI

struct OptimizationView: View {
    @EnvironmentObject var vm: MainViewModel
    @State private var pending: Suggestion?

    private var suggestions: [Suggestion] { vm.advice?.suggestions ?? [] }

    var body: some View {
        Group {
            if suggestions.isEmpty {
                VStack(spacing: 14) {
                    Image(systemName: "wand.and.stars")
                        .font(.system(size: 44))
                        .foregroundColor(.textFaint)
                    Text("暂无优化建议")
                        .font(.title2.weight(.semibold))
                        .foregroundColor(.textPrimary)
                    Text("系统健康分 \(vm.report?.healthScore ?? 100)，无需优化")
                        .foregroundColor(.textSecondary)
                }
                .frame(maxWidth: .infinity, maxHeight: .infinity)
                .background(Color.appBackground)
            } else {
                List {
                    Section {
                        ForEach(suggestions) { suggestion in
                            SuggestionRow(suggestion: suggestion) {
                                run(suggestion)
                            }
                        }
                    } header: {
                        Text(vm.advice?.rootCauseSummary ?? "")
                    } footer: {
                        if let result = vm.lastAction {
                            VStack(alignment: .leading, spacing: 4) {
                                Label(
                                    result.success ? "完成" : "未完成",
                                    systemImage: result.success ? "checkmark.circle.fill" : "exclamationmark.triangle.fill"
                                )
                                .foregroundColor(result.success ? .success : .warning)
                                Text(result.message)
                                    .foregroundColor(.textSecondary)
                            }
                            .padding(.top, 8)
                        }
                    }
                }
                .background(Color.appBackground)
            }
        }
        .navigationTitle("优化中心")
        .alert(item: $pending) { suggestion in
            Alert(
                title: Text("确认执行"),
                message: Text("\(suggestion.reason)\n\n风险级别：\(suggestion.riskLevel.display)。\n执行前请确认已保存工作。"),
                primaryButton: .destructive(Text("执行")) {
                    Task { await vm.execute(suggestion) }
                },
                secondaryButton: .cancel(Text("取消"))
            )
        }
    }

    private func run(_ suggestion: Suggestion) {
        if suggestion.requiresConfirm {
            pending = suggestion
        } else {
            Task { await vm.execute(suggestion) }
        }
    }
}

struct SuggestionRow: View {
    let suggestion: Suggestion
    let onExecute: () -> Void

    @EnvironmentObject var vm: MainViewModel

    private var color: Color {
        switch suggestion.riskLevel {
        case .high: return .severityHigh
        case .medium: return .severityMedium
        case .low: return .severityLow
        }
    }

    var body: some View {
        HStack(alignment: .center, spacing: 12) {
            VStack(alignment: .leading, spacing: 4) {
                HStack(spacing: 8) {
                    Text(suggestion.actionType.rawValue)
                        .font(.system(.caption, design: .monospaced))
                        .padding(.horizontal, 6)
                        .padding(.vertical, 2)
                        .background(color.opacity(0.15))
                        .clipShape(Capsule())
                    Text(suggestion.riskLevel.display)
                        .font(.caption)
                        .foregroundColor(color)
                    if suggestion.requiresConfirm {
                        Image(systemName: "lock.fill")
                            .font(.caption2)
                            .foregroundColor(.warning)
                    }
                }
                Text(suggestion.reason)
                    .font(.subheadline)
                    .foregroundColor(.textPrimary)
            }

            Spacer()

            Button("执行", action: onExecute)
                .buttonStyle(.borderedProminent)
                .tint(suggestion.riskLevel == .high ? .danger : .accent)
                .disabled(vm.isExecuting)
        }
        .padding(.vertical, 4)
    }
}