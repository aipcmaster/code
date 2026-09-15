// 智能诊断：问题列表（严重度降序，来自诊断引擎）。

import SwiftUI

struct DiagnosisView: View {
    @EnvironmentObject var vm: MainViewModel

    private var issues: [Issue] { vm.report?.issues ?? [] }

    var body: some View {
        Group {
            if issues.isEmpty {
                VStack(spacing: 14) {
                    Image(systemName: "checkmark.shield")
                        .font(.system(size: 44))
                        .foregroundColor(.success)
                    Text("一切正常")
                        .font(.title2.weight(.semibold))
                        .foregroundColor(.textPrimary)
                    Text("未检测到需要关注的问题")
                        .foregroundColor(.textSecondary)
                }
                .frame(maxWidth: .infinity, maxHeight: .infinity)
                .background(Color.appBackground)
            } else {
                List {
                    ForEach(Array(issues.enumerated()), id: \.offset) { _, issue in
                        IssueRow(issue: issue)
                    }
                }
                .background(Color.appBackground)
            }
        }
        .navigationTitle("智能诊断")
    }
}

struct IssueRow: View {
    let issue: Issue

    private var color: Color {
        switch issue.severity {
        case .critical: return .severityCritical
        case .high: return .severityHigh
        case .medium: return .severityMedium
        case .low: return .severityLow
        }
    }

    var body: some View {
        HStack(alignment: .top, spacing: 12) {
            Circle()
                .fill(color)
                .frame(width: 10, height: 10)
                .padding(.top, 5)

            VStack(alignment: .leading, spacing: 4) {
                HStack(spacing: 8) {
                    Text(issue.title)
                        .font(.headline)
                        .foregroundColor(.textPrimary)
                    Text("· \(issue.category.display)")
                        .font(.subheadline)
                        .foregroundColor(.textFaint)
                }
                Text(issue.description)
                    .font(.subheadline)
                    .foregroundColor(.textSecondary)
                    .fixedSize(horizontal: false, vertical: true)
                Text("严重度：\(issue.severity.display)")
                    .font(.caption2)
                    .foregroundColor(color)
            }

            Spacer()
        }
        .padding(.vertical, 4)
    }
}