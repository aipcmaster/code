# AIPCMaster（AI电脑大师）核心引擎 — core-rust

![license](https://img.shields.io/badge/license-proprietary-blue)

对应《AIPCMaster（AI电脑大师）软件开发工程文档》§12 目录结构中的 `core-rust/`：
端侧高性能系统操作与跨平台核心。只读，不做任何系统修改。

## 当前内容

| Crate | 说明 | 对应文档章节 |
| - | - | - |
| `aipcmaster-collect` | 数据采集层：CPU/内存/磁盘/网络/温度/进程 | SD §2.1、PRD §4.4 |
| `aipcmaster-diagnose` | 诊断引擎：健康分 + 问题检测规则集 | SD §3.1、PRD US-02、ERD §3.10 |
| `aipcmaster-advisor` | AI 推理层：根因分析 + 优化建议（规则化实现，可换 LLM） | SD §2.2、ERD §3.11 |
| `aipcmaster-store` | 本地存储：SQLite 快照/会话/报告/审计日志 | ERD §3.4-3.9 |
| `aipcmaster-sampler` | 采样调度：采集→存储→诊断→频率自适应（5s/500ms） | SD §2.1、PRD §5 |

## 设计原则

- **跨平台 trait + 平台后端**：`SystemCollector` trait 定义契约，Linux 后端读取
  `/proc`、`/sys`（零守护进程依赖）；Windows/macOS 后端后续通过各自原生 API 实现。
- **只读**：本层只做采集，系统修改属于决策执行层（优化执行模块）职责。
- **可测试**：所有 `/proc` 解析为纯函数，用 fixture 文本做单元测试（不依赖真实主机）；
  另有 `tests/smoke.rs` 在真实主机上做结构冒烟校验。
- **采样节奏常量**：正常 5 秒（`NORMAL_SAMPLE_INTERVAL`），异常 500 毫秒
  （`ABNORMAL_SAMPLE_INTERVAL`），PRD §5 硬性要求。

## 使用方法

```bash
cargo test                                   # 单元 + 冒烟测试
cargo run -p aipcmaster-collect --example snapshot          # 轻量快照(JSON)
cargo run -p aipcmaster-collect --example snapshot -- --full # 完整快照(含进程/磁盘IO)
cargo run -p aipcmaster-diagnose --example diagnose          # 采集+现场诊断(健康分/问题)
```

## 数据来源（Linux）

| 指标 | 来源 |
| - | - |
| CPU 使用率（整体+每核） | `/proc/stat` 两次采样差分 |
| CPU 负载均值 | `/proc/loadavg` |
| CPU 温度 | `/sys/class/thermal/thermal_zone*/temp`（取最高） |
| 内存 | `/proc/meminfo`（MemTotal/MemAvailable/Swap） |
| 磁盘容量 | `/proc/mounts` + `statvfs(3)` |
| 磁盘 I/O | `/proc/diskstats`（累计计数） |
| 网络 | `/proc/net/dev`（累计计数） |
| 进程 | `/proc/<pid>/stat` + `statm`（内存降序 Top 32） |

单位约定：字节一律 SI 字节；百分比 0.0~100.0；时间 Unix 毫秒。

## 下一步（路线图）

1. **采样调度服务**：`tokio` 定时任务，正常/异常采样切换，超时与错峰。
2. **速率计算**：对两次快照求差得出磁盘/网络速率（供诊断与仪表盘）。
3. **AI 推理层接入**：本地 LLM/轻量模型对规则结果做根因分析与自然语言报告。
4. **优化执行层**：建议 → 用户确认 → 还原点 → 执行 → 回滚（决策执行层、SD 2.3）。
5. **Windows/macOS 后端**：`cfg(target_os)` 平台分支。
6. **本地存储**：采集数据本地缓存（断网可用）。

## 参考

- 技术栈与架构：SD §1、§2
- MVP 范围：PRD §2.1
- 非功能需求：PRD §5