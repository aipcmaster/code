# AIPCMaster（AI电脑大师）

AI 驱动的电脑优化与维护助手。本项目包含全部产品文档与各端实现（monorepo，对齐《软件开发工程文档》§12 目录布局）。

## 仓库结构

```
AIPCMaster/
├── core-rust/         核心引擎（Rust，跨平台）
│   ├── crates/aipcmaster-collect   数据采集层（Linux /proc + /sys）
│   ├── crates/aipcmaster-diagnose  诊断引擎（健康分 + 规则）
│   ├── crates/aipcmaster-store     本地存储（SQLite，ERD 对齐）
│   └── crates/aipcmaster-sampler   采样调度（正常 5s / 异常 500ms）
├── server/            云端服务（Rust axum REST API 网关）
├── web-console/       云端控制台（零依赖单页应用）
├── web/               官网（已上线 https://aipcmaster.com）
├── web-src/           官网生成器（content.py + generate.py）
├── .github/workflows/ CI（fmt · clippy · test）
└── *.md               产品文档（PRD / SD / ERD / 时序图 / BP，位于仓库根）
```

## 快速开始

### 核心引擎（Linux）

```bash
cd core-rust
cargo test --workspace          # 采集 + 诊断 + 存储 + 采样全量测试
cargo run --example diagnose    # 当前机器一键诊断 → 健康分 + 问题列表
```

### 云端服务

```bash
cd server
python3 serve.py                 # 启动（127.0.0.1:8787）
# 打开 http://127.0.0.1:8787/ 即控制台；API 文档见 server/README.md
```

### 控制台

`web-console/index.html` 零依赖，由 server 直接提供（`/` 与 `/console`）。

## 架构（SD §2）

| 层 | 实现 | 说明 |
| - | - | - |
| 数据采集层 | `aipcmaster-collect` + `aipcmaster-sampler` | 5s 正常 / 500ms 异常采样，本地 SQLite 落库 |
| AI 推理层 | `aipcmaster-diagnose`（规则引擎） | 健康分 0-100 + 可插拔规则（后续接 LLM 根因分析） |
| 决策执行层 | server optimizations API | suggested→executed→rolled_back 状态机，高危须确认 |
| 用户交互层 | `web-console` + `web` | 控制台 + 官网 |

## 平台状态

| 组件 | 状态 |
| - | - |
| core-rust | ✅ 可编译可测试（Linux） |
| server | ✅ 可编译可测试，真实 HTTP 冒烟通过 |
| web-console | ✅ 对接真实 API |
| client-windows | 📋 待实现（需 .NET 8 + WinUI 3 环境） |
| client-macos | 📋 待实现（需 macOS + Xcode） |

## 文档

产品设计文档（PRD / 软件开发工程文档 / ERD / 核心时序图 / BP）位于项目根目录，是各模块实现的唯一依据。

## 许可

见根目录 LICENSE（商业软件，保留所有权利）。