# AIPCMaster 云端服务（aipcmaster-server）

对齐《AIPCMaster（AI电脑大师）软件开发工程文档》§4 API 设计 与 §5 数据存储 的云端服务实现。

## 特性

- **统一返回结构**（SD §4.1）：`{code, message, data, request_id}`
- **核心 API（SD §4.2）**：
  - `POST /api/v1/auth/register` · `/login` · `/refresh`
  - `GET /api/v1/users/me`
  - `GET/POST /api/v1/devices` · `POST /api/v1/devices/register` · `DELETE /api/v1/devices/{id}`
  - `POST /api/v1/diagnostics/sessions` · `GET /api/v1/diagnostics/reports/{id}`
  - `POST /api/v1/optimizations/actions` · `.../actions/{id}/execute` · `.../logs/{id}/rollback`
  - `GET /api/v1/subscriptions/current` · `POST /api/v1/subscriptions/checkout`
  - `GET /api/v1/alerts` · `POST /api/v1/api-keys` · `GET /api/v1/audit-logs` · `POST /api/v1/support/tickets`
- **错误码（SD §4.4）**：40001/40101/40301/40401/40901/42901/50001/60001/60002
- **认证**：JWT（HS256）+ Refresh Token + PBKDF2 密码哈希（生产建议 argon2id/jsonwebtoken，见代码注释）
- **数据**：SQLite（开发/演示）——表结构对齐 ERD §3，生产切换 PostgreSQL + Redis
- **安全**：免费版设备数限制（60002）、高危优化动作须确认（40001）、API Key 哈希存储

## 安全配置（环境变量）

| 变量 | 默认 | 说明 |
| - | - | - |
| `AIPCMASTER_JWT_SECRET` | 无（**必填**） | JWT 签名密钥，≥32 字符强随机。缺失/过弱时服务**拒绝启动**（fail-closed）。生成：`openssl rand -hex 32` |
| `AIPCMASTER_INSECURE_DEV` | 关闭 | 设为 `1` 允许使用开发默认密钥（**仅本地**，会打印告警） |
| `AIPCMASTER_ALLOW_MOCK_CHECKOUT` | 关闭 | 设为 `1` 启用模拟支付（联调演示；**生产必须关闭**，否则用户可自助升级 Pro） |
| `AIPCMASTER_CORS_ORIGINS` | 本机来源 | 逗号分隔的允许来源白名单；`*` 放开全部（不推荐） |
| `AIPCMASTER_ADDR` / `AIPCMASTER_DB` | `127.0.0.1:8787` / `aipcmaster.db` | 监听地址 / 数据库路径 |

本地开发快捷方式：`python3 serve.py`（自动设置 `AIPCMASTER_INSECURE_DEV=1` 与 `AIPCMASTER_ALLOW_MOCK_CHECKOUT=1`）。

### 已实现的安全控制（CSO 审计后）

- **弱密钥 fail-closed**：生产未配置强密钥则拒绝启动，杜绝默认密钥令牌伪造
- **恒定时间比较**：JWT 签名与密码校验均为常量时间
- **登录/注册限流**：滑动窗口（登录 10 次/5 分钟、注册 5 次/小时，按邮箱）
- **恒时化登录**：用户不存在时也执行一次 PBKDF2，防响应时间枚举邮箱
- **刷新重读权限**：刷新令牌时从库重读 role/plan，降权立即生效
- **输入长度上限**：邮箱/密码/设备名/工单等均有上限
- **IDOR 防护**：所有 `{id}` 资源均按 `user_id` 归属校验
- **SQL 参数化**：全量 `params![]`，无字符串拼接
- **API Key 哈希存储**：明文仅返回一次

## 运行

```bash
# 开发模式（默认 127.0.0.1:8787，SQLite 文件 aipcmaster.db）
AIPCMASTER_INSECURE_DEV=1 AIPCMASTER_ALLOW_MOCK_CHECKOUT=1 cargo run

# 生产（强密钥 + 真实支付 + 白名单 CORS）
AIPCMASTER_ADDR=0.0.0.0:8787 AIPCMASTER_DB=/var/lib/aipcmaster/app.db \
AIPCMASTER_JWT_SECRET=$(openssl rand -hex 32) \
AIPCMASTER_CORS_ORIGINS=https://console.aipcmaster.com \
cargo run --release
```

## 测试

```bash
cargo test          # 单元 + 集成（真实 HTTP 层，tower oneshot）
cargo clippy        # 零警告
```

集成测试覆盖：注册/登录/刷新、40101 未认证、设备注册与 60002 超限、诊断会话归属校验、
优化动作状态机（suggested→executed→rolled_back）、高危动作确认、API Key 哈希落库、
订阅升级、工单创建；**安全回归**：登录限流 42901、模拟支付默认禁用 40301、
刷新重读角色、超长输入拒绝。

## 布局

```
src/
├── auth.rs        JWT 签发/校验 + PBKDF2 + 弱密钥判定 + 恒时化
├── rate_limit.rs  滑动窗口限流（登录/注册）
├── error.rs       统一响应 + 错误码
├── state.rs       AppState + SQLite 迁移（ERD 表投影）
└── routes/        auth · users · devices · diagnostics · optimizations ·
                   subscriptions · alerts · api_keys · audit · support
```

## 生产差异（TODO）

- PostgreSQL 替换 SQLite；Redis 会话/限流（当前为进程内内存限流）
- **IP 维度限流**：应用层限流按邮箱；需在反向代理/网关叠加按 IP 限流
- OAuth2/OIDC 接入；argon2id
- 支付渠道（微信/支付宝/Stripe）替换模拟 checkout，并校验回调签名
- 刷新令牌轮换/吊销列表（当前为无状态 JWT，30 天有效期内不可单独吊销）
- 注册接口的邮箱枚举（409「已注册」）——生产建议改为邮件验证的通用响应
- 设备 mTLS / 证书（SD §6）