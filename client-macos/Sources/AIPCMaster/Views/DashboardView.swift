// 仪表盘：健康分环 + CPU/内存/磁盘指标。

import SwiftUI

struct DashboardView: View {
    @EnvironmentObject var vm: MainViewModel

    private var score: Int { vm.report?.healthScore ?? 100 }
    private var cpuPercent: Double { vm.snapshot?.cpu?.usagePercent ?? 0 }
    private var memPercent: Double { vm.snapshot?.memory?.usagePercent ?? 0 }
    private var diskPercent: Double { vm.snapshot?.disks.max(by: { $0.usagePercent < $1.usagePercent })?.usagePercent ?? 0 }
    private var cores: Int { vm.snapshot?.cpu?.coreCount ?? 0 }
    private var loadAvg: (Double, Double, Double)? {
        guard let c = vm.snapshot?.cpu, let l1 = c.loadAvg1, let l5 = c.loadAvg5, let l15 = c.loadAvg15 else { return nil }
        return (l1, l5, l15)
    }

    var body: some View {
        ScrollView {
            VStack(spacing: 28) {
                HealthRing(score: score)

                HStack(spacing: 16) {
                    MetricCard(title: "CPU", percent: cpuPercent, systemImage: "cpu")
                    MetricCard(title: "内存", percent: memPercent, systemImage: "memorychip")
                    MetricCard(title: "磁盘", percent: diskPercent, systemImage: "internaldrive")
                }

                if let load = loadAvg {
                    VStack(spacing: 6) {
                        Text("系统负载").font(.subheadline).foregroundColor(.textSecondary)
                        Text(String(format: "1m %.2f    5m %.2f    15m %.2f    (%d 核)", load.0, load.1, load.2, cores))
                            .font(.system(.body, design: .monospaced))
                    }
                    .padding(.top, 4)
                }

                Text("采样 \(vm.sampleCount) 次 · 正常 5s / 异常 0.5s")
                    .font(.caption2)
                    .foregroundColor(.textFaint)
            }
            .padding(24)
            .frame(maxWidth: 520)
            .frame(maxWidth: .infinity)
        }
        .background(Color.appBackground)
        .navigationTitle("仪表盘")
    }
}

/// 健康分环形进度（100 为满环）。
struct HealthRing: View {
    let score: Int

    var body: some View {
        ZStack {
            Circle()
                .stroke(Color.textFaint.opacity(0.15), lineWidth: 14)
            Circle()
                .trim(from: 0, to: CGFloat(score) / 100)
                .stroke(
                    AngularGradient(colors: [.accent, .success], center: .center),
                    style: StrokeStyle(lineWidth: 14, lineCap: .round)
                )
                .rotationEffect(.degrees(-90))
            VStack(spacing: 2) {
                Text("\(score)")
                    .font(.system(size: 44, weight: .bold, design: .rounded))
                    .foregroundColor(.textPrimary)
                Text("健康分")
                    .font(.caption)
                    .foregroundColor(.textSecondary)
            }
        }
        .frame(width: 170, height: 170)
    }
}

/// 指标卡片（百分比 + 进度条）。
struct MetricCard: View {
    let title: String
    let percent: Double
    let systemImage: String

    private var color: Color {
        switch percent {
        case 90...: return .danger
        case 75..<90: return .warning
        default: return .success
        }
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 10) {
            Label(title, systemImage: systemImage)
                .font(.headline)
                .foregroundColor(.textPrimary)
            Text(String(format: "%.1f%%", percent))
                .font(.system(size: 26, weight: .semibold, design: .rounded))
                .foregroundColor(color)
            ProgressView(value: percent, total: 100)
                .tint(color)
        }
        .padding(16)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(Color.appCard)
        .clipShape(RoundedRectangle(cornerRadius: 12))
        .overlay(
            RoundedRectangle(cornerRadius: 12)
                .stroke(Color.textFaint.opacity(0.12), lineWidth: 1)
        )
    }
}