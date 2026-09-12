// AIPCMaster（AI电脑大师）Windows 客户端

对应《AIPCMaster（AI电脑大师）软件开发工程文档》§12 目录结构中的 `client-windows/`：
Windows 端用户界面与应用逻辑（SD §2.4 用户交互层 + §2.3 决策执行层）。

## 技术栈

- **.NET 8 + WPF**（SD §1：Windows 客户端 C#/.NET 8 + WinUI 3 或 WPF；选 WPF：LTS 稳定、部署简单）
- **零第三方 NuGet 包**（离线可构建）：采集层用 P/Invoke（kernel32），JSON 用 System.Text.Json，HTTP 用 System.Net.Http

## 目录结构

```
client-windows/
├── src/
│   ├── AIPCMaster.Core/          # 纯逻辑类库（可独立测试、跨 UI 复用）
│   │   ├── Collectors/           # 采集层：WindowsSystemCollector（P/Invoke）
│   │   ├── Diagnosis/            # 诊断引擎 + 健康分（阈值对齐 core-rust）
│   │   ├── Advisor/              # AI 推理层：根因分析 + 优化建议（ERD 3.11）
│   │   └── Optimization/         # 决策执行层：执行器 + 审计日志（US-03）
│   └── AIPCMaster.App/           # WPF UI（MVVM 手写基架）
│       ├── Views/                # 仪表盘 / 智能诊断 / 优化中心 / 设置
│       ├── ViewModels/           # MainViewModel（全链路编排）
│       └── Services/             # ApiClient（对接 server）、LocalSettings
└── tests/
    └── ConsoleSelfCheck/         # 自检程序（无需测试框架）
```

## 构建（需 Windows + .NET 8 SDK）

```powershell
# 全链路自检（采集→诊断→建议→健康分算法）
dotnet run --project tests/ConsoleSelfCheck

# 构建桌面应用
dotnet build src/AIPCMaster.App

# 发布单目录
dotnet publish src/AIPCMaster.App -c Release -r win-x64 --self-contained true
```

> 本仓库开发环境为 Linux（无 .NET SDK），代码经静态审视 + 逻辑自检程序对齐
> core-rust 已测试实现；首次构建请在 Windows 机器执行。若发现编译问题，
> 提交前请修正（预期为 XAML 细节类问题，逻辑层与 core-rust 51 测试对齐）。

## 云端对接（server：axum REST API）

| 客户端动作 | 调用的服务端 API |
| - | - |
| 注册 / 登录 | `POST /api/v1/auth/register` / `login`（统一响应 §4.1） |
| 设备绑定 | `POST /api/v1/devices/register`（PRD §4.4 首次启动生成设备 ID） |
| 诊断上报（预留） | `POST /api/v1/diagnostics/sessions` |
| 优化记录 | `POST /api/v1/optimizations/actions`（状态机 suggested→executed） |
| 订阅状态 | `GET /api/v1/subscriptions/current` |

ApiClient 内置：401 自动用 refresh token 刷新重试；API 地址可在「设置」页修改。

## 架构对齐（SD §2）

| SD 层 | 本实现 | 说明 |
| - | - | - |
| 数据采集层 | `WindowsSystemCollector` | P/Invoke GetSystemTimes（CPU 双采样差分）/ GlobalMemoryStatusEx（内存）/ DriveInfo（磁盘）。温度/负载 Windows 无公开 API → null，规则自动跳过（不误报） |
| AI 推理层 | `DiagnosticEngine` + `Advisor` | 阈值与 core-rust 完全一致：CPU>90% High、内存>90%/75% High/Medium、磁盘>95%/85% Critical/High、温度>95°/80° Critical/High、Swap>50% Medium |
| 决策执行层 | `OptimizationExecutor` + `AuditLogger` | 默认只读；高危确认（US-03）；磁盘清理创建还原点（SRSetRestorePoint）；审计 JSONL 落盘 %LOCALAPPDATA%/AIPCMaster/logs |
| 用户交互层 | WPF 四页 | 仪表盘（健康环）/ 诊断 / 优化 / 设置 |

## 平台说明

- 采样节奏：正常 5s / 异常 500ms 高频（PRD §5 硬性，MainViewModel.PollTick 实现）
- 健康分算法：前 3 严重问题扣分（Critical40/High25/Medium15/Low5），下限 20（core-rust health.rs 同构）
- 高危动作执行前弹确认框；磁盘清理自动创建系统还原点兜底

## 遗留（V1.1+）

- CPU 温度采集：需接入 WMI/MSR 驱动（当前 null）
- 启动项管理：需管理员权限（V1 只读提示）
- 安全扫描：V1 占位（LLM 接入后增强，SD §2.2）