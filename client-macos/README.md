# AIPCMaster（AI电脑大师）macOS 客户端

对应《AIPCMaster（AI电脑大师）软件开发工程文档》§12 目录结构中的 `client-macos/`：
macOS 端用户界面与应用逻辑（SD §2.4 用户交互层 + §2.3 决策执行层）。
与 `client-windows/`（.NET 8 + WPF）功能对等。

## 技术栈

- **SwiftUI + Swift Package**（macOS 13+，SD §1：macOS 客户端 Swift/SwiftUI）
- **零第三方依赖**（离线可构建）：采集用系统 API（Darwin/Mach sysctl），
  无 SPM 远程包 —— 与 Windows 版"零 NuGet"约束一致

## 目录结构

```
client-macos/
├── Package.swift                    # SPM 清单，4 个 target
├── Sources/
│   ├── AIPCMasterCore/              # 纯逻辑层（可独立测试、跨 UI 复用）
│   │   ├── Collectors/              # MacSystemCollector（Darwin/Mach 采集）
│   │   ├── Diagnosis/               # 诊断引擎 + 健康分（阈值对齐 core-rust）
│   │   ├── Advisor/                 # AI 推理层：根因分析 + 建议（ERD 3.11）
│   │   └── Optimization/            # 决策执行层：执行器 + 审计日志（US-03）
│   ├── AIPCMaster/                  # SwiftUI 桌面应用
│   │   ├── Views/                   # 仪表盘 / 智能诊断 / 优化中心 / 设置
│   │   ├── Services/                # ApiClient（预留）、LocalSettings
│   │   └── MainViewModel.swift      # 全链路编排（5s/500ms 采样）
│   ├── AIPCMasterSelfCheck/         # 自检程序（无 UI，只读）
│   └── (tests 见 Tests/)
├── Tests/AIPCMasterCoreTests/       # XCTest 单测（健康分/诊断/建议）
└── scripts/make-app.sh              # 打包最小 .app（无需 Xcode 工程）
```

## 构建（需 macOS 13+，Xcode 15 或 Swift 5.9+ 工具链）

```bash
# 单元测试（健康分已知用例 / 诊断阈值 / Advisor 去重升序）
swift test

# 全链路自检（采集→诊断→建议→健康分算法，只读）
swift run AIPCMasterSelfCheck

# 运行桌面应用
swift run AIPCMaster

# 打包 .app（最小应用包，双击可运行）
bash scripts/make-app.sh && open build/AIPCMaster.app
```

> 本仓库开发环境为 Linux（无 Swift 工具链），代码经静态审视 + 逻辑自检
> 对齐 client-windows（已对齐 core-rust 已测试实现）；首次构建请在 Mac 执行。
> 若发现编译问题，提交前请修正（预期为极少数 swiftc 类型推断细节，逻辑层一致）。

## 采集实现（与 Windows 版对照）

| 指标 | Windows（P/Invoke） | macOS（系统 API） |
| - | - | - |
| CPU | `GetSystemTimes` 双采样差分 | `host_processor_info` tick 双采样差分（含每核） |
| 内存 | `GlobalMemoryStatusEx` | `host_statistics64`（active+wire+compressed） |
| Swap | 页面文件计数 | `sysctl vm.swapusage`（无 swap → 规则跳过） |
| 磁盘 | `DriveInfo`（Fixed） | `statvfs` 遍历挂载卷（过滤系统内部卷） |
| 负载 | 无 API → nil，规则跳过 | **`getloadavg`（真实值，负载规则生效）** |
| 温度 | 无公开 API → nil | SMC 需 root → nil，规则跳过（不误报） |

跨平台语义一致：字段为 nil 时对应规则自动跳过（Rust Option 同构）。

## 优化动作（macOS 版）

| 动作 | Windows | macOS | 风险 / 需确认 |
| - | - | - | - |
| clean_memory | 清空工作集（无权限） | `purge`（osascript 授权框） | 低/中；Critical 需确认 |
| disk | 删 %TEMP% 超 30 天 + 系统还原点 | **移入废纸篓**（可恢复）超 30 天缓存 | 中/高；High 需确认 |
| startup | 只读（需管理员） | 只读（V1 占位） | 低 |
| generic | 占位 | 占位（安全扫描 V1.1） | 高；需确认 |

安全性差异：macOS 无系统还原点，磁盘清理**移入废纸篓而非删除**（比 Windows 更安全，可恢复）。
高危动作执行前弹确认框（SD §2.3 默认只读）；审计日志 JSONL 落盘
`~/Library/Application Support/AIPCMaster/logs/`。

## 云端对接（预留，默认关闭）

| 客户端动作 | 服务端 API |
| - | - |
| 设备标识 | `POST /api/v1/devices/register`（首次启动生成持久 UUID，PRD §4.4） |
| 优化记录 | `POST /api/v1/optimizations/actions`（状态机 suggested→executed） |

「设置」页开启同步后为尽力而为上报，失败不影响本地功能。

## 平台说明

- 采样节奏：正常 5s / 异常 500ms 高频（PRD §5 硬性，MainViewModel 实现）
- 健康分：前 3 严重问题扣分（Critical40/High25/Medium15/Low5），下限 20（core-rust 同构）
- 可视化令牌对齐官网调色板（text-faint #7D879B 等）

## 遗留（V1.1+）

- CPU 温度采集：需 SMC 特权（`powermetrics`），当前 nil
- 启动项管理：需系统权限（V1 只读提示）
- 安全扫描：V1 占位（LLM 接入后增强，SD §2.2）
- 应用签名/公证：本地分发无需，公开分发需 Developer ID + notarization