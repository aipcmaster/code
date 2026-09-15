// AIPCMaster（AI电脑大师）macOS 客户端 —— 应用入口。
// SwiftUI TabView 四页，与 Windows WPF 四页对应：仪表盘 / 智能诊断 / 优化中心 / 设置。

import SwiftUI

@main
struct AIPCMasterApp: App {
    @StateObject private var vm = MainViewModel()

    var body: some Scene {
        WindowGroup {
            ContentView()
                .environmentObject(vm)
                .onAppear { vm.start() }
                .frame(minWidth: 780, minHeight: 540)
        }
        .windowResizability(.contentMinSize)
    }
}

struct ContentView: View {
    @EnvironmentObject var vm: MainViewModel

    var body: some View {
        TabView {
            DashboardView()
                .tabItem { Label("仪表盘", systemImage: "gauge") }
            DiagnosisView()
                .tabItem { Label("智能诊断", systemImage: "stethoscope") }
            OptimizationView()
                .tabItem { Label("优化中心", systemImage: "wand.and.stars") }
            SettingsView()
                .tabItem { Label("设置", systemImage: "gearshape") }
        }
    }
}