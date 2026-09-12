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

## 运行

```bash
# 开发模式（默认 127.0.0.1:8787，SQLite 文件 aipcmaster.db）
cargo run

# 配置（环境变量）
AIPCMASTER_ADDR=0.0.0.0:8787 AIPCMASTER_DB=/var/lib/aipcmaster/app.db \
AIPCMASTER_JWT_SECRET=$(openssl rand -hex 32) cargo run --release
```

## 测试

```bash
cargo test          # 单元 + 集成（真实 HTTP 层，tower oneshot）
cargo clippy        # 零警告
```

集成测试覆盖：注册/登录/刷新、40101 未认证、设备注册与 60002 超限、诊断会话归属校验、
优化动作状态机（suggested→executed→rolled_back）、高危动作确认、API Key 哈希落库、
订阅升级、工单创建。

## 布局

```
src/
├── auth.rs        JWT 签发/校验 + PBKDF2
├── error.rs       统一响应 + 错误码
├── state.rs       AppState + SQLite 迁移（ERD 表投影）
└── routes/        auth · users · devices · diagnostics · optimizations ·
                   subscriptions · alerts · api_keys · audit · support
```

## 生产差异（TODO）

- PostgreSQL 替换 SQLite；Redis 会话/限流
- OAuth2/OIDC 接入；argon2id
- 支付渠道（微信/支付宝/Stripe）替换模拟 checkout
- 设备 mTLS / 证书（SD §6）