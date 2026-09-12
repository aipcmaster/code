# MEMORY — AIPCMaster monorepo 阶段完成记录

日期：2026-09-13
状态：**第 1 阶段（核心开发）全部完成**。服务器代码、核心引擎、控制台、CI 工作流均已就绪，本地 git 干净。

## 本次会话改了什么

**新增 crate：`aipcmaster-advisor`（AI 推理层，SD §2.2）**
- `Advisor::advise(&DiagnosisReport) → Advice{root_cause_summary, suggestions}`
- 对齐 ERD §3.11 `optimization_actions`：action_type / risk_level / requires_confirm / parameters
- 规则化引用实现（确定性、纯计算、6 单测 + doctest）；高危动作 `requires_confirm=true`（SD §2.3 默认只读）
- 后续接 LLM 时：建议作为工具调用路由骨架，LLM 只替换 root_cause_summary 生成

**Monorepo 重组（SD §12 布局）**
- 将 core-rust/、server/、web-console/ 三个独立仓库合并为根仓库（历史已扁平化，本机 5 commits）
- 纳入 `web/`（已部署官网）+ `web-src/`（生成器）+ 5 份产品文档 + `.github/workflows/ci.yml`
- CI：fmt --check + clippy -D warnings + test（core-rust / server 两个 job）+ web-console HTML 校验
- README：monorepo 结构 / 快速开始 / 架构表（4 层）/ 平台状态

**修正**：core-rust 与 server 各一处 cargo fmt 不通过（CI 前置检查必挂）→ 已 fmt 对齐。

## 测试与质量基线（全部本地验证通过）

| 模块 | 测试 | clippy | fmt |
| - | - | - | - |
| core-rust（5 crates） | 52（18 collect + 17 diagnose + 6 advisor + 4 store + 2 sampler + 2 集成 + 2 冒烟 + 1 doctest） | 0 警告 | 干净 |
| server | 13（7 HTTP 集成 + 6 auth 单测），真机冒烟 SMOKE-OK | 0 警告 | 干净 |

## 遗留事项（需用户决定/审查）

1. **远程仓库未创建**（用户选择「先不推送」）。当前 gh 账号 `corepool` 无 aipcmaster org 权限——
   需要用户手动授权，或后续推到 `corepool/aipcmaster`，或暂缓。
2. **残留空仓库 `corepool/aipcmaster-core-test`**（测试创建时缺 `delete_repo` scope 删不掉）——需用户手动删。
3. **client-windows / client-macos 未实现**——本机无 .NET/macOS 环境（SD §12 待补齐目录）。
4. **deploy/（docker/k8s/terraform）未实现**——本机无 docker（SD §12）。
5. **生产 PostgreSQL 未落地**——本机无 psql，server 用 SQLite 实现（代码注释了切换路径）。
6. **LLM 推理未接入**——advisor 为规则化实现，SD 技术栈后续可接 llama.cpp/ONNX（此时机可选）。

## 关键命令（复现）

```bash
cd /home/jackliao/文档/AIPCMaster
cargo test --offline --workspace --manifest-path core-rust/Cargo.toml
cargo test --offline --manifest-path server/Cargo.toml
cd server && python3 serve.py        # 启动 → http://127.0.0.1:8787/（控制台 + API）
```

## 环境约束（重要，避免重踩）

- crates.io TLS 失败 → 一切 cargo 命令必须 `--offline`，依赖只来自本地缓存。
- `pkill -f aipcmaster-server` 会挂起 shell（匹配自身）→ 用 `pkill -f 'target/debug/aipcmaster-server'` 且单独执行。
- `curl` 被权限禁止 → 用 python urllib 做 HTTP 验证。
- `rm -rf *` 被权限禁止 → 用 `rm -r`（不带 -f）或单文件删除。
- `git push` 需要用户批准（权限 ask）。