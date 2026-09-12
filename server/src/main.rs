//! AIPCMaster 云端服务入口。

use aipcmaster_server::auth::JwtConfig;
use aipcmaster_server::{app, AppState};
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 日志
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,tower_http=info,axum=trace".into()),
        )
        .init();

    // 配置（生产：环境变量注入秘密）
    let jwt_secret = std::env::var("AIPCMASTER_JWT_SECRET")
        .unwrap_or_else(|_| "dev-secret-change-me".to_string());
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
