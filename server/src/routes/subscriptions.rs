//! 订阅路由（ERD 3.5 subscriptions / 3.6 orders）。

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::{ApiResponse, AppState};
use axum::extract::State;
use axum::Json;
use rusqlite::OptionalExtension;
use serde::Deserialize;
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CheckoutRequest {
    pub plan: String, // pro
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}

/// GET /api/v1/subscriptions/current —— 当前订阅。
pub async fn current(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    let conn = state.db.lock().unwrap();
    let row: Option<(String, String, Option<i64>, i64)> = conn
        .query_row(
            "SELECT id, plan, expires_at, created_at FROM subscriptions \
             WHERE user_id=?1 AND status='active' ORDER BY created_at DESC LIMIT 1",
            rusqlite::params![auth.user_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .optional()
        .map_err(ApiError::from)?;

    // 无订阅→默认 free
    let sub = row.unwrap_or_else(|| (String::new(), "free".to_string(), None, now_ms()));

    let app_plan = if sub.1 == "free" {
        "free"
    } else if sub.2.map(|e| e > now_ms()).unwrap_or(false) {
        "pro"
    } else {
        "expired"
    };

    let req_id = crate::routes::next_request_id();
    Ok(Json(ApiResponse::with_data(
        json!({
            "plan": app_plan,
            "status": if app_plan == "pro" { "active" } else { "none" },
            "expires_at": sub.2,
            "created_at": sub.3,
        }),
        &req_id,
    )))
}

/// POST /api/v1/subscriptions/checkout —— 创建支付（此处为模拟：直接激活 pro 订阅）。
/// 生产接入微信/支付宝/Stripe；本实现用于联调演示。
pub async fn checkout(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<CheckoutRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    if body.plan != "pro" {
        return Err(ApiError::bad_request("当前仅支持 pro 订阅"));
    }
    let conn = state.db.lock().unwrap();
    let now = now_ms();
    let expires = now + 30 * 24 * 3600 * 1000; // 30 天

    // 将旧订阅置为 inactive
    conn.execute(
        "UPDATE subscriptions SET status='canceled' WHERE user_id=?1 AND status='active'",
        rusqlite::params![auth.user_id],
    )
    .map_err(ApiError::from)?;

    let sub_id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO subscriptions(id, user_id, plan, status, expires_at, created_at) \
         VALUES(?1, ?2, 'pro', 'active', ?3, ?4)",
        rusqlite::params![sub_id, auth.user_id, expires, now],
    )
    .map_err(ApiError::from)?;

    // 模拟订单
    let order_id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO orders(id, user_id, amount_cents, status, created_at) \
         VALUES(?1, ?2, 1990, 'paid', ?3)",
        rusqlite::params![order_id, auth.user_id, now],
    )
    .map_err(ApiError::from)?;

    let req_id = crate::routes::next_request_id();
    Ok(Json(ApiResponse::with_data(
        json!({
            "subscription_id": sub_id,
            "order_id": order_id,
            "plan": "pro",
            "expires_at": expires,
        }),
        &req_id,
    )))
}
