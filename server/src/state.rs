//! 应用状态：SQLite 连接（生产切 PostgreSQL）+ JWT 配置。

use crate::auth::JwtConfig;
use crate::rate_limit::RateLimiter;
use rusqlite::Connection;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// 应用共享状态。
#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Mutex<Connection>>,
    pub jwt: JwtConfig,
    /// 免费版允许的最大设备数。
    pub free_device_limit: i64,
    /// 是否允许模拟支付（联调演示用）。生产必须为 false，否则用户可自助免费升级 Pro。
    pub allow_mock_checkout: bool,
    /// 登录限流（防暴力破解）。
    pub login_limiter: Arc<RateLimiter>,
    /// 注册限流（防批量注册）。
    pub register_limiter: Arc<RateLimiter>,
}

impl AppState {
    /// 使用文件数据库（或 `:memory:`）。
    pub fn new(path: impl AsRef<Path>, jwt: JwtConfig) -> rusqlite::Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL;").ok();
        // 生产默认关闭模拟支付；仅当显式设置 AIPCMASTER_ALLOW_MOCK_CHECKOUT=1 时启用
        let allow_mock_checkout = matches!(
            std::env::var("AIPCMASTER_ALLOW_MOCK_CHECKOUT").as_deref(),
            Ok("1") | Ok("true") | Ok("yes")
        );
        let state = Self {
            db: Arc::new(Mutex::new(conn)),
            jwt,
            free_device_limit: 2,
            allow_mock_checkout,
            login_limiter: Arc::new(RateLimiter::new(10, Duration::from_secs(300))),
            register_limiter: Arc::new(RateLimiter::new(5, Duration::from_secs(3600))),
        };
        state.migrate()?;
        Ok(state)
    }

    /// 内存数据库（测试）。
    pub fn in_memory() -> rusqlite::Result<Self> {
        let mut state = Self::new(":memory:", JwtConfig::default())?;
        // 测试环境启用模拟支付，便于覆盖订阅流程
        state.allow_mock_checkout = true;
        Ok(state)
    }

    /// 建表（ERD §3 各表的开发版 SQLite 投影）。
    pub fn migrate(&self) -> rusqlite::Result<()> {
        let conn = self.db.lock().unwrap();
        conn.execute_batch(
            r#"
            PRAGMA foreign_keys = ON;

            -- ERD 3.1 users
            CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY,
                email TEXT UNIQUE NOT NULL,
                password_hash TEXT NOT NULL,
                display_name TEXT,
                role TEXT NOT NULL DEFAULT 'member',
                created_at INTEGER NOT NULL
            );

            -- ERD 3.2 organizations
            CREATE TABLE IF NOT EXISTS organizations (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                owner_user_id TEXT NOT NULL,
                created_at INTEGER NOT NULL
            );

            -- ERD 3.3 organization_members
            CREATE TABLE IF NOT EXISTS organization_members (
                id TEXT PRIMARY KEY,
                org_id TEXT NOT NULL REFERENCES organizations(id),
                user_id TEXT NOT NULL REFERENCES users(id),
                role TEXT NOT NULL DEFAULT 'member',
                created_at INTEGER NOT NULL
            );

            -- ERD 3.4 devices
            CREATE TABLE IF NOT EXISTS devices (
                id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL REFERENCES users(id),
                org_id TEXT REFERENCES organizations(id),
                device_name TEXT NOT NULL,
                os TEXT,
                version TEXT,
                status TEXT NOT NULL DEFAULT 'active',
                last_seen_at INTEGER,
                created_at INTEGER NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_devices_user ON devices(user_id);

            -- ERD 3.5 subscriptions
            CREATE TABLE IF NOT EXISTS subscriptions (
                id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL REFERENCES users(id),
                plan TEXT NOT NULL DEFAULT 'free',
                status TEXT NOT NULL DEFAULT 'active',
                expires_at INTEGER,
                created_at INTEGER NOT NULL
            );

            -- ERD 3.6 orders
            CREATE TABLE IF NOT EXISTS orders (
                id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL REFERENCES users(id),
                amount_cents INTEGER NOT NULL,
                status TEXT NOT NULL DEFAULT 'pending',
                created_at INTEGER NOT NULL
            );

            -- ERD 3.9 diagnostic_sessions（云端脱敏摘要）
            CREATE TABLE IF NOT EXISTS diagnostic_sessions (
                id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL REFERENCES users(id),
                device_id TEXT REFERENCES devices(id),
                trigger_type TEXT NOT NULL,
                status TEXT NOT NULL,
                created_at INTEGER NOT NULL
            );

            -- ERD 3.10 diagnostic_reports（云端只存脱敏摘要）
            CREATE TABLE IF NOT EXISTS diagnostic_reports (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL REFERENCES diagnostic_sessions(id),
                device_id TEXT REFERENCES devices(id),
                health_score INTEGER NOT NULL,
                summary TEXT,
                issues_json TEXT,
                created_at INTEGER NOT NULL
            );

            -- ERD 3.11 optimization_actions
            CREATE TABLE IF NOT EXISTS optimization_actions (
                id TEXT PRIMARY KEY,
                issue_id TEXT,
                device_id TEXT NOT NULL REFERENCES devices(id),
                action_type TEXT NOT NULL,
                risk_level TEXT NOT NULL,
                requires_confirm INTEGER NOT NULL DEFAULT 1,
                parameters_json TEXT,
                status TEXT NOT NULL DEFAULT 'suggested',
                created_at INTEGER NOT NULL
            );

            -- ERD 3.12 optimization_logs
            CREATE TABLE IF NOT EXISTS optimization_logs (
                id TEXT PRIMARY KEY,
                action_id TEXT NOT NULL REFERENCES optimization_actions(id),
                device_id TEXT NOT NULL,
                user_id TEXT NOT NULL REFERENCES users(id),
                executed_at INTEGER NOT NULL,
                result TEXT NOT NULL,
                rollback_status TEXT NOT NULL DEFAULT 'none',
                log_text TEXT
            );

            -- ERD 3.14 alerts
            CREATE TABLE IF NOT EXISTS alerts (
                id TEXT PRIMARY KEY,
                device_id TEXT NOT NULL REFERENCES devices(id),
                user_id TEXT REFERENCES users(id),
                org_id TEXT REFERENCES organizations(id),
                type TEXT NOT NULL,
                severity TEXT NOT NULL,
                title TEXT NOT NULL,
                message TEXT,
                status TEXT NOT NULL DEFAULT 'unread',
                created_at INTEGER NOT NULL
            );

            -- ERD 3.15 audit_logs
            CREATE TABLE IF NOT EXISTS audit_logs (
                id TEXT PRIMARY KEY,
                actor_user_id TEXT NOT NULL REFERENCES users(id),
                org_id TEXT REFERENCES organizations(id),
                action TEXT NOT NULL,
                resource_type TEXT NOT NULL,
                resource_id TEXT,
                ip TEXT,
                user_agent TEXT,
                created_at INTEGER NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_audit_actor ON audit_logs(actor_user_id, created_at);

            -- ERD 3.16 api_keys
            CREATE TABLE IF NOT EXISTS api_keys (
                id TEXT PRIMARY KEY,
                user_id TEXT REFERENCES users(id),
                org_id TEXT REFERENCES organizations(id),
                name TEXT NOT NULL,
                key_hash TEXT NOT NULL,
                scopes_json TEXT NOT NULL DEFAULT '[]',
                last_used_at INTEGER,
                expires_at INTEGER,
                status TEXT NOT NULL DEFAULT 'active',
                created_at INTEGER NOT NULL
            );

            -- ERD 3.17 support_tickets
            CREATE TABLE IF NOT EXISTS support_tickets (
                id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL REFERENCES users(id),
                org_id TEXT REFERENCES organizations(id),
                device_id TEXT REFERENCES devices(id),
                subject TEXT NOT NULL,
                content TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'open',
                created_at INTEGER NOT NULL
            );
            "#,
        )
    }
}
