//! AIPCMaster（AI电脑大师）云端服务
//!
//! 对齐《软件开发工程文档》§4 API 设计：
//! - 统一返回结构：`{code, message, data, request_id}`（§4.1）；
//! - 核心 API 列表（§4.2）；
//! - JWT + Refresh Token（§4.3）；
//! - 错误码（§4.4）。
//!
//! 注：本机无 PostgreSQL，开发用 SQLite（rusqlite），生产部署切换为
//! PostgreSQL + Redis（见 SD §5 数据存储），路由与模型层不变。

pub mod auth;
pub mod error;
pub mod rate_limit;
pub mod routes;
pub mod state;

pub use error::{ApiError, ApiResponse};
pub use state::AppState;

/// 应用入口：构建路由与状态（供 main 与集成测试共用）。
pub fn app(state: AppState) -> axum::Router {
    routes::router(state)
}
