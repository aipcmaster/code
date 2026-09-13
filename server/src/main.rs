//! AIPCMaster 云端服务入口。

use aipcmaster_server::auth::{is_weak_secret, JwtConfig};
use aipcmaster_server::{app, AppState};
use std::net::SocketAddr;

/// 加载 JWT 密钥：生产必须显式配置强密钥，否则拒绝启动。
///
/// 安全要求（防止令牌伪造）：未设置 `AIPCMASTER_JWT_SECRET`、或密钥过短 /
/// 为已知默认值时，服务**拒绝启动**（fail-closed）。本地开发如确需默认值，
/// 显式设置 `AIPCMASTER_INSECURE_DEV=1`，此时会打印醒目告警。
fn load_jwt_secret() -> anyhow::Result<String> {
    let insecure_dev = matches!(
        std::env::var("AIPCMASTER_INSECURE_DEV").as_deref(),
        Ok("1") | Ok("true") | Ok("yes")
    );

    match std::env::var("AIPCMASTER_JWT_SECRET") {
        Ok(secret) if !is_weak_secret(&secret) => Ok(secret),
        Ok(secret) => {
            if insecure_dev {
                tracing::warn!(
                    "⚠ 使用弱 JWT 密钥（AIPCMASTER_INSECURE_DEV=1，仅限本地开发，切勿用于生产）"
                );
                Ok(secret)
            } else {
                anyhow::bail!(
                    "AIPCMASTER_JWT_SECRET 过弱或为已知默认值（需 ≥32 字符的强随机密钥）。\n\
                     生成：openssl rand -hex 32\n\
                     本地开发如确需默认值：设置 AIPCMASTER_INSECURE_DEV=1"
                )
            }
        }
        Err(_) => {
            if insecure_dev {
                tracing::warn!(
                    "⚠ 未设置 AIPCMASTER_JWT_SECRET，使用开发默认密钥（AIPCMASTER_INSECURE_DEV=1，切勿用于生产）"
                );
                Ok(JwtConfig::default().secret)
            } else {
                anyhow::bail!(
                    "未设置 AIPCMASTER_JWT_SECRET（生产必须显式配置强随机密钥）。\n\
                     生成：openssl rand -hex 32\n\
                     本地开发：设置 AIPCMASTER_INSECURE_DEV=1 可使用默认密钥"
                )
            }
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 日志
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,tower_http=info,axum=trace".into()),
        )
        .init();

    // 配置（生产：环境变量注入强密钥；弱密钥 fail-closed）
    let jwt_secret = load_jwt_secret()?;
    let db_path = std::env::var("AIPCMASTER_DB").unwrap_or_else(|_| "aipcmaster.db".to_string());
    let listen_addr: SocketAddr = std::env::var("AIPCMASTER_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:8787".to_string())
        .parse()
        .expect("AIPCMASTER_ADDR 必须为 socket 地址");

    let jwt = JwtConfig {
        secret: jwt_secret,
        ..Default::default()
    };
    let state = AppState::new(&db_path, jwt)?;
    let app = app(state);

    let listener = tokio::net::TcpListener::bind(listen_addr).await?;
    tracing::info!("AIPCMaster 服务启动: http://{listen_addr}  (db: {db_path})");
    axum::serve(listener, app).await?;
    Ok(())
}
