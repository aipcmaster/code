//! 用户路由。

use crate::auth::AuthUser;
use crate::error::ApiError;
use rusqlite::OptionalExtension;
use crate::{AppState, ApiResponse};
use axum::extract::State;
use axum::Json;
use serde_json::json;

/// 通用用户查询实体。
pub struct UserRow {
    pub email: String,
    pub display_name: String,
    pub role: String,
    pub created_at: i64,
}

/// 按 ID 查用户，不存在返回 None。
pub fn find_user(state: &AppState, user_id: &str) -> Result<Option<UserRow>, ApiError> {
    let conn = state.db.lock().unwrap();
    let row = conn
        .query_row(
            "SELECT email, display_name, role, created_at FROM users WHERE id=?1",
            rusqlite::params![user_id],
            |r| {
                Ok(UserRow {
                    email: r.get(0)?,
                    display_name: r.get(1)?,
                    role: r.get(2)?,
                    created_at: r.get(3)?,
                })
            },
        )
        .optional()
        .map_err(ApiError::from)?;
    Ok(row.map(|r| UserRow {
        email: r.email,
        display_name: r.display_name,
        role: r.role,
        created_at: r.created_at,
    }))
}

/// GET /api/v1/users/me
pub async fn me(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    let user = find_user(&state, &auth.user_id)?
        .ok_or_else(|| ApiError::not_found("用户不存在"))?;

    // plan
    let plan: String = state
        .db
        .lock()
        .unwrap()
        .query_row(
            "SELECT plan FROM subscriptions WHERE user_id=?1 AND status='active' \
             ORDER BY created_at DESC LIMIT 1",
            rusqlite::params![auth.user_id],
            |r| r.get::<_, String>(0),
        )
        .optional()
        .map_err(ApiError::from)?
        .unwrap_or_else(|| "free".to_string());

    let data = json!({
        "id": auth.user_id,
        "email": user.email,
        "display_name": user.display_name,
        "role": user.role,
        "plan": plan,
        "created_at": user.created_at,
    });
    let req_id = crate::routes::next_request_id();
    Ok(Json(ApiResponse::with_data(data, &req_id)))
}

#[allow(dead_code)]
fn unused(_: axum::http::HeaderMap) {}