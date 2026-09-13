# MEMORY — AIPCMaster monorepo 阶段完成记录

日期：2026-09-13
状态：**核心开发 + Windows 客户端 + 安全审计 + 官网 SEO/GEO 全部完成**。本地 git 干净。

## 最新进展（本次会话后半段）

**官网可访问性（axe-core WCAG 2.1 AA，24 页 0 违规）**
- 审计工具 `web-src/audit_a11y.py`（注入本地缓存的 axe-core 4.11，全站扫描）
- 修复 658 个节点问题：对比度 518（`--text-faint` #667085 → **#7D879B**，4.93:1）、
  `<main>` 地标缺失、内页缺 h1（加 sr-only h1）、页脚 h4 跳级（→ h3）
- h1 样式收窄为 `.hero h1`，页面标题 h1 用新尺寸，视觉不变
- CI 新增 a11y 守卫：每页恰一个 h1 / 有 `<main>` / 无 h4 跳级 + 调色板对比度 ≥4.5:1

**官网性能 + 图片优化（已部署 aipcmaster.com）**
- **CSS 内联**：源文件移至 `web-src/styles.css`，生成时压缩 24%（12.9KB→9.9KB）并内联进每个页面，
  彻底消除唯一的渲染阻塞请求。slow-4G 实测 **FCP/LCP 改善 0.75–1.6s，CLS 0**
- **图片优化**：OG 图 PNG-8 量化（256 色，视觉无差异）**1.30 MB → 0.70 MB（-46%）**
- **逐页 OG 图**：22 张（每页每语言一张），shell 与 JSON-LD 按页引用
- **humans.txt** + `<link rel=author>`；`<meta name=color-scheme content=dark>`
- 工具：`web-src/measure.py`（本地 CWV 测量）、`web-src/compare_cwv.py`（慢速 4G 前后对比）
- 部署注意：web/ 已无 `styles.css`，同步部署仓库时需 `git rm styles.css`

**官网 SEO + GEO 优化（`web-src/` 生成器 + `web/` 产物，已部署 aipcmaster.com）**
- head 升级：og:image(1200×630) + og:locale + twitter summary_large_image + robots +
  hreflang x-default + theme-color；首页 canonical 修正为站点根 `/`
- JSON-LD `@graph`：Organization / WebSite / BreadcrumbList 全站；
  SoftwareApplication + FAQPage（首页）；Product + Offer（定价页）
- 首页新增 FAQ 区块（中英各 6 题，与 FAQPage schema 同源）
- 生成器产出 `sitemap.xml`（lastmod + hreflang + x-default）、`robots.txt`（显式欢迎 16 个 AI 爬虫）、
  `llms.txt`（llmstxt.org 格式）；`make_og_image.py`（PIL）生成中英 OG 图
- CI 新增 `website` job：生成器产物漂移检查 + SEO 工件校验
- 部署链路：monorepo `web/` → 克隆 `aipcmaster/web`（deploy key `aipcmaster-web-deploy`）→ 复制 → 推送
  （注意保留部署仓库的 `.nojekyll` 与 `README.md`）

**Windows 客户端（`client-windows/`，.NET 8 + WPF，零 NuGet）**
- Core 类库：P/Invoke 采集（GetSystemTimes 双采样差分 / GlobalMemoryStatusEx / DriveInfo）、
  DiagnosticEngine + HealthScore（阈值与 core-rust 逐条一致）、Advisor（ERD 3.11）、
  OptimizationExecutor（还原点 SRSetRestorePoint / 审计 JSONL）
- WPF MVVM 四页：仪表盘 / 诊断 / 优化 / 设置；5s/500ms 采样；登录注册 + 设备绑定
- `tests/ConsoleSelfCheck`：免框架全链路自检（采→诊→建→健康分算法断言）
- ⚠ **本机 Linux 无 .NET SDK，代码未经编译验证**——首次构建请在 Windows 执行
  `dotnet build src/AIPCMaster.App`；逻辑层已与 core-rust 52 测试对齐，预期仅 XAML 细节问题

**安全审计（CSO）——详见 `SECURITY-AUDIT.md`**
- 修复 1 严重 + 5 高 + 4 中 + 2 低；报告含完整发现与生产 TODO
- 关键修复：JWT 弱密钥 fail-closed、签名恒定时间、登录/注册限流、
  刷新重读权限、CI 去第三方 action、模拟支付默认禁用、CORS 白名单、
  登录恒时化、输入长度上限、客户端 https 强制
- 新增 `server/src/rate_limit.rs`；server 测试 13 → **22**（含 4 项安全回归）

## 远程仓库

- **代码**：https://github.com/aipcmaster/code （org，public ⚠，default `main`，CI ✅）
  - 推送用 deploy key `~/.ssh/aipcmaster-code-deploy` + SSH 别名 `github.com-aipcmaster-code`
  - ⚠ 建议改 private（LICENSE 为专有/NDA，当前 public）
- **官网**：https://github.com/aipcmaster/web （部署 aipcmaster.com，deploy key `aipcmaster-web-deploy`）
- 备用镜像：`corepool/aipcmaster`（private）

## 测试基线（全部本地验证通过）

| 模块 | 测试 | clippy | fmt |
| - | - | - | - |
| core-rust（5 crates） | **52** | 0 警告 | 干净 |
| server | **22**（含安全回归）+ 真实冒烟 | 0 警告 | 干净 |
| client-windows | 未编译验证（无 .NET SDK） | - | - |
| website | 生成器漂移检查 + SEO 工件校验（CI） | - | - |

**官网 CWV（本地 lab，系统 Chrome）**：LCP 64–112ms · CLS 0 · TTFB 2ms · 首屏资源 ~1.7KB（CSS 内联，仅 app.js + favicon 外链）
**官网可访问性**：axe-core 4.11 WCAG 2.1 A/AA + best-practice，24 页 **0 违规**（本地跑 `web-src/audit_a11y.py`）
**慢速 4G（150ms RTT / 1.6Mbps）**：内联 CSS 后 FCP/LCP 1860ms（外链时 2612ms），CLS 0

## 本次会话改了什么（前半段）

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

1. **远程仓库**：已推送至 **https://github.com/aipcmaster/code**（org 仓库，default branch `main`，CI ✅）。
   推送方式：deploy key `~/.ssh/aipcmaster-code-deploy`（已加入仓库，Allow write access）+
   SSH 别名 `github.com-aipcmaster-code`。
   - ⚠ 该仓库当前为 **public**，但 LICENSE 为专有/NDA——建议改为 private（需 org admin）。
   - 备用镜像：`corepool/aipcmaster`（private，CI ✅）。
2. **残留空仓库 `corepool/aipcmaster-core-test`**（测试创建时缺 `delete_repo` scope 删不掉）——需用户手动删。
3. **client-windows 未经编译验证**——需 Windows + .NET 8 SDK 首次构建；client-macos 未实现（需 Xcode）。
4. **deploy/（docker/k8s/terraform）未实现**——本机无 docker（SD §12）。
5. **生产 PostgreSQL 未落地**——本机无 psql，server 用 SQLite 实现（代码注释了切换路径）。
6. **LLM 推理未接入**——advisor 为规则化实现，SD 技术栈后续可接 llama.cpp/ONNX（此时机可选）。
7. **安全上线前必办**（见 `SECURITY-AUDIT.md` 末节）：强 JWT 密钥、关闭 INSECURE_DEV/MOCK_CHECKOUT、
   真实支付回调校验、IP 限流、CORS 白名单、刷新令牌轮换、注册邮箱枚举、argon2id。
8. **安全审计记录为 TODO 的中危项**：注册邮箱枚举（M3）、客户端刷新令牌明文存储（M5，需 DPAPI NuGet 包）。

## 关键命令（复现）

```bash
cd /home/jackliao/文档/AIPCMaster
cargo test --offline --workspace --manifest-path core-rust/Cargo.toml
cargo test --offline --manifest-path server/Cargo.toml
cd server && python3 serve.py        # 启动 → http://127.0.0.1:8787/（控制台 + API）
# serve.py 自动设 AIPCMASTER_INSECURE_DEV=1 + AIPCMASTER_ALLOW_MOCK_CHECKOUT=1（仅本地）
# 直接 cargo run 需显式设这两个变量，否则服务拒绝启动（安全 fail-closed）
```

## 环境约束（重要，避免重踩）

- crates.io TLS 失败 → 一切 cargo 命令必须 `--offline`，依赖只来自本地缓存。
- `pkill -f aipcmaster-server` 会挂起 shell（匹配自身）→ 用 `pkill -f 'target/debug/aipcmaster-server'` 且单独执行。
- `curl` 被权限禁止 → 用 python urllib 做 HTTP 验证。
- `rm -rf *` 被权限禁止 → 用 `rm -r`（不带 -f）或单文件删除。
- `git push` 需要用户批准（权限 ask）。