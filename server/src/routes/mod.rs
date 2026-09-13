//! API 路由装配（SD §4.2 核心 API）。

use crate::{ApiResponse, AppState};
use axum::extract::Request;
use axum::middleware::{self, Next};
use axum::response::Response;
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use std::sync::atomic::{AtomicU64, Ordering};

pub mod alerts;
pub mod api_keys;
pub mod audit;
pub mod auth;
pub mod devices;
pub mod diagnostics;
pub mod optimizations;
pub mod subscriptions;
pub mod support;
pub mod users;

/// 请求 ID 计数器。
static REQ_COUNTER: AtomicU64 = AtomicU64::new(0);

/// 生成请求 ID：`req_<unix>_<counter>`。
pub fn next_request_id() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    format!(
        "req_{now:016x}_{}",
        REQ_COUNTER.fetch_add(1, Ordering::Relaxed)
    )
}

/// 请求 ID（可注入扩展，供 handler 读取统一响应）。
#[derive(Debug, Clone)]
pub struct RequestId(pub String);

/// 注入请求 ID 的中间件。
async fn request_context(req: Request, next: Next) -> Response {
    let request_id = next_request_id();
    let mut req = req;
    req.extensions_mut().insert(RequestId(request_id.clone()));
    let mut res = next.run(req).await;
    let header: axum::http::HeaderValue = request_id
        .parse()
        .unwrap_or_else(|_| axum::http::HeaderValue::from_static("req"));
    res.headers_mut().insert("x-request-id", header);
    res
}

/// 未匹配路由兜底。
async fn fallback() -> Json<ApiResponse<serde_json::Value>> {
    Json(ApiResponse::ok(&next_request_id()))
}

/// 静态控制台：GET /console 与 / 提供 web-console/index.html。
/// 零依赖实现（避免 tower-http fs 的额外依赖）。
pub async fn console() -> impl axum::response::IntoResponse {
    use axum::http::{header, HeaderValue};
    let path = std::env::var("AIPCMASTER_CONSOLE_PATH")
        .unwrap_or_else(|_| "../web-console/index.html".to_string());
    match tokio::fs::read(&path).await {
        Ok(bytes) => {
            let mut res = axum::response::Response::new(axum::body::Body::from(bytes));
            res.headers_mut().insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static("text/html; charset=utf-8"),
            );
            res
        }
        Err(_) => axum::response::Response::new(axum::body::Body::from(
            "控制台文件未找到（设置 AIPCMASTER_CONSOLE_PATH）",
        )),
    }
}

/// 构建 CORS 层。
///
/// 默认仅允许本机来源（控制台与 API 同源，跨域仅用于本地联调）。
/// 通过 `AIPCMASTER_CORS_ORIGINS` 覆盖：逗号分隔的来源列表，或 `*` 放开全部。
/// 放开全部仅适用于无 Cookie 的 Bearer 令牌 API，生产建议显式白名单。
fn cors_layer() -> tower_http::cors::CorsLayer {
    use axum::http::{HeaderValue, Method};
    use tower_http::cors::{AllowOrigin, Any, CorsLayer};

    let origins = std::env::var("AIPCMASTER_CORS_ORIGINS").unwrap_or_default();
    let origin_cfg = if origins.trim() == "*" {
        AllowOrigin::any()
    } else if origins.trim().is_empty() {
        AllowOrigin::list(
            [
                "http://127.0.0.1:8787",
                "http://localhost:8787",
                "http://127.0.0.1:5173",
                "http://localhost:5173",
            ]
            .iter()
            .filter_map(|o| HeaderValue::from_str(o).ok()),
        )
    } else {
        AllowOrigin::list(
            origins
                .split(',')
                .filter_map(|o| HeaderValue::from_str(o.trim()).ok()),
        )
    };

    CorsLayer::new()
        .allow_origin(origin_cfg)
        .allow_methods([Method::GET, Method::POST, Method::DELETE, Method::PUT])
        .allow_headers(Any)
}

/// 组装完整路由。
pub fn router(state: AppState) -> Router {
    let api = Router::new()
        .route("/auth/register", post(auth::register))
        .route("/auth/login", post(auth::login))
        .route("/auth/refresh", post(auth::refresh))
        .route("/users/me", get(users::me))
        .route("/devices", get(devices::list))
        .route("/devices/register", post(devices::register))
        .route("/devices/{id}", delete(devices::unregister))
        .route("/diagnostics/sessions", post(diagnostics::create_session))
        .route("/diagnostics/reports/{id}", get(diagnostics::get_report))
        .route(
            "/optimizations/actions/{id}/execute",
            post(optimizations::execute),
        )
        .route("/optimizations/actions", post(optimizations::propose))
        .route(
            "/optimizations/logs/{id}/rollback",
            post(optimizations::rollback),
        )
        .route("/subscriptions/current", get(subscriptions::current))
        .route("/subscriptions/checkout", post(subscriptions::checkout))
        .route("/alerts", get(alerts::list))
        .route("/api-keys", post(api_keys::create))
        .route("/audit-logs", get(audit::list))
        .route("/support/tickets", post(support::create_ticket))
        .fallback(fallback);

    Router::new()
        .nest("/api/v1", api)
        .route("/", get(console))
        .route("/console", get(console))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            request_context,
        ))
        .layer(cors_layer())
        .with_state(state)
}
