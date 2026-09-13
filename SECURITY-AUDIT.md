# AIPCMaster 安全审计报告（CSO）

日期：2026-09-13
范围：monorepo 全量（core-rust / server / client-windows / web-console / web / CI）
模式：daily（8/10 置信度门槛）
方法：CSO 审计流程（攻击面测绘 → 密钥考古 → 供应链 → CI/CD → 代码/OWASP → 修复验证）

---

## 摘要

| 严重度 | 数量 | 已修复 |
| - | - | - |
| 严重（Critical） | 1 | 1 |
| 高（High） | 5 | 5 |
| 中（Medium） | 6 | 4（2 项记录为生产 TODO） |
| 低（Low） | 5 | 2（3 项记录） |

**结论**：修复后服务端具备生产可用的安全基线。核心链路（JWT 签名、密码哈希、SQL、IDOR、XSS）
经审计确认无结构性缺陷；主要风险集中在**默认配置**与**缺失的防护层**，均已修复或明确记录。

---

## 攻击面测绘

```
代码面
  公开端点（未认证）:      3   （register / login / refresh）
  需认证端点:              13
  资源 {id} 端点:          4   （devices / diagnostics / optimizations×2）
  静态控制台:              2   （/ 与 /console，读 env 指定路径，非用户输入）
基础设施面
  CI/CD 工作流:            1   （.github/workflows/ci.yml）
  容器/IaC:                0
  .env 入库:               0   （.gitignore 已排除）
  密钥管理:                环境变量（AIPCMASTER_JWT_SECRET）
```

栈：Rust（axum 0.8 · rusqlite · hmac/sha2/pbkdf2）· .NET 8 WPF（C#）· 原生 HTML/JS · GitHub Actions

---

## 发现与处置

### 严重（Critical）

**C1 — JWT 密钥回退到硬编码默认值** · `server/src/main.rs:18`
未设置 `AIPCMASTER_JWT_SECRET` 时回退到仓库内公开的 `"dev-secret-change-me"`。
任何读到源码的人可伪造任意用户（含 admin）令牌 → 完全账户接管。
**修复**：改为 fail-closed——缺失/过弱（<32 字符或已知默认值）时**拒绝启动**；
新增 `is_weak_secret()` 与 `AIPCMASTER_INSECURE_DEV` 显式逃生舱（仅本地）。
**验证**：无密钥/弱密钥启动均被拒绝（见 smoke 测试）。

### 高（High）

**H1 — JWT 签名非恒定时间比较** · `server/src/auth.rs:104`
原用 `expected != actual` 切片比较（短路），注释却声称"恒定时间"——HMAC 校验的时序侧信道。
**修复**：改用 `hmac::Mac::verify_slice()`（常量时间）。

**H2 — 认证端点无速率限制** · `server/src/routes/auth.rs`
登录可被暴力破解/撞库，注册可被批量创建。
**修复**：新增 `rate_limit.rs` 滑动窗口限流器；登录 10 次/5 分钟、注册 5 次/小时（按邮箱）。
超限返回 42901（SD §4.4 已定义但此前从未使用）。
**验证**：集成测试 `login_rate_limited_after_max_attempts`。

**H3 — 刷新令牌不轮换 + 权限陈旧** · `server/src/routes/auth.rs:refresh`
30 天刷新令牌不可吊销；刷新时沿用旧令牌中的 role/plan，降权/订阅到期不生效。
**修复**：刷新时从库重读 role 与 plan 再签发。
**验证**：集成测试 `refresh_rereads_role_from_db`。
（轮换/吊销列表需状态存储，列为生产 TODO。）

**H4 — CI 使用未固定的第三方 action** · `.github/workflows/ci.yml:23,55`
`dtolnay/rust-toolchain@stable` 指向可移动 tag，供应链攻击面（CI 持有仓库 token）。
**修复**：移除该第三方 action，改用 runner 预装工具链 + `rustup component add`。

**H5 — 模拟支付可被任意用户自助升级** · `server/src/routes/subscriptions.rs:checkout`
`checkout` 直接激活 Pro，无支付校验 → 生产环境收入漏洞。
**修复**：默认禁用（返回 40301）；仅 `AIPCMASTER_ALLOW_MOCK_CHECKOUT=1` 时启用。
**验证**：集成测试 `mock_checkout_disabled_by_default`。

### 中（Medium）

**M1 — CORS 通配** · `server/src/routes/mod.rs` — `allow_origin(Any)` + `allow_headers(Any)`。
**修复**：默认仅本机来源白名单；`AIPCMASTER_CORS_ORIGINS` 可配（`*` 显式放开）。

**M2 — 时序枚举邮箱** · `server/src/routes/auth.rs:login` — 用户不存在时立即返回，密码错误则跑 10 万轮 PBKDF2，响应时间可区分。
**修复**：新增 `equalize_password_timing()`，不存在路径也执行一次 PBKDF2。

**M3 — 注册接口枚举邮箱** · `server/src/routes/auth.rs:register` — 409「该邮箱已注册」可枚举。
**处置**：**记录为生产 TODO**（正确修法是邮件验证 + 通用响应；当前为联调阶段保留 UX）。

**M4 — 输入无长度上限** · devices / support / api_keys / auth。
**修复**：邮箱 254、密码 8–128、显示名 64、设备名 64、OS 64、版本 32、工单主题 200/正文 10000、Key 名 64。

**M5 — 刷新令牌明文落盘** · `client-windows/.../LocalSettings.cs` — 存于用户级 `%LOCALAPPDATA%` 明文 JSON。
**处置**：**记录**（正确修法是 DPAPI 加密，需 `System.Security.Cryptography.ProtectedData` NuGet 包，离线环境不可用）+ 代码注释说明与缓解。

**M6 — 客户端允许非 HTTPS API 地址** · `client-windows/.../MainViewModel.cs` — 令牌可能明文经网络传输。
**修复**：新增 `IsAcceptableApiUrl()`——非本机地址强制 https，保存时校验。

### 低（Low）

| # | 发现 | 处置 |
| - | - | - |
| L1 | 邮箱校验偏弱（仅含 `@`） | 已加去空白校验；更严格需 RFC 5322/验证邮件 |
| L2 | 密码仅校验长度 | 记录（生产接 zxcvbn，SD 已注明） |
| L3 | web-console 内联 `onclick` 拼接 `dev.id` | 记录；`dev.id` 为服务端 UUID，非用户输入 |
| L4 | 未知路由 fallback 返回 200 | 记录；不影响安全 |
| L5 | 工单 `device_id` 未校验归属 | **已修复**：校验设备属于当前用户 |

---

## 审计确认无缺陷的项

- **SQL 注入**：全量 `params![]` 参数化，无字符串拼接 SQL（server + core-rust store）
- **IDOR**：所有 `{id}` 资源均 JOIN 归属表并按 `user_id` 过滤（devices/diagnostics/optimizations）
- **认证覆盖**：除 3 个公开认证端点外，全部路由强制 `AuthUser` 提取器
- **XSS**：web-console 对 API 数据统一 `esc()` 转义（`& < > " '`），错误/提示用 `textContent`
- **密钥考古**：git 历史无 AKIA/ghp_/sk-/私钥等凭据；无 `.env` 入库；无硬编码生产凭据
- **JWT alg 混淆**：验签前校验 `alg == HS256`，无 `alg:none` 绕过
- **API Key**：PBKDF2 哈希落库，明文仅返回一次
- **密码哈希**：PBKDF2-HMAC-SHA256 10 万轮 + 随机盐 + 恒定时间比较
- **Rust unsafe**：仅 Linux `statvfs` 一处，NUL 已由 `CString::new().ok()?` 处理，指针合法

---

## 验证

```
server:    22 tests passed（含 4 项安全回归）· clippy 0 警告 · fmt 干净
core-rust: 52 tests passed
真实冒烟： fail-closed 拒绝弱密钥；超长输入 40001；限流 42901；dev 模式 checkout 可用
```

## 生产上线前必办（TODO）

1. `AIPCMASTER_JWT_SECRET=$(openssl rand -hex 32)`（否则拒绝启动）
2. 不设置 `AIPCMASTER_INSECURE_DEV` / `AIPCMASTER_ALLOW_MOCK_CHECKOUT`
3. 接入真实支付并校验回调签名
4. 反向代理层叠加按 IP 限流
5. 配置 `AIPCMASTER_CORS_ORIGINS` 为控制台正式域名
6. 评估：刷新令牌轮换/吊销、注册邮箱枚举、argon2id、PostgreSQL、设备 mTLS
