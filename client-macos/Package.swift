// swift-tools-version:5.9
// AIPCMaster（AI电脑大师）macOS 客户端 —— Swift Package，零第三方依赖。
// 与 Windows 客户端（.NET 8 + WPF，零 NuGet）架构对齐：Core 纯逻辑 + App(SwiftUI)。

import PackageDescription

let package = Package(
    name: "AIPCMaster",
    platforms: [
        .macOS(.v13), // Ventura+（2022）：SwiftUI TabView/Charts 稳定基线
    ],
    targets: [
        // 纯逻辑层（采集/诊断/建议/执行），独立于 UI，可单测
        .target(name: "AIPCMasterCore"),

        // SwiftUI 桌面应用
        .executableTarget(
            name: "AIPCMaster",
            dependencies: ["AIPCMasterCore"]
        ),

        // 自检程序（无 UI，命令行运行，对应 Windows ConsoleSelfCheck）
        .executableTarget(
            name: "AIPCMasterSelfCheck",
            dependencies: ["AIPCMasterCore"]
        ),

        // 单元测试（macOS 上 swift test 运行）
        .testTarget(
            name: "AIPCMasterCoreTests",
            dependencies: ["AIPCMasterCore"]
        ),
    ]
)