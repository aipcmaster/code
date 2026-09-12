//! 认证路由：注册 / 登录 / 刷新。

use crate::auth::{issue_tokens, validate_refresh, AuthUser};
use crate::error::ApiError;
use rusqlite::OptionalExtension;
use crate::{AppState, ApiResponse};
use axum::extract::State;
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub display_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub user: serde_json::Value,
    pub tokens: crate::auth::TokenPair,
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}

fn valid_email(email: &str) -> bool {
    let email = email.trim();
    !email.is_empty() && email.contains('@') && email.len() <= 254
}

/// 校验密码强度（生产建议结合 zxcvbn；这里为最小合理校验）。
fn valid_password(pw: &str) -> bool {
    pw.len() >= 8
}

/// POST /api/v1/auth/register
pub async fn register(
    State(state): State<AppState>,
    Json(body): Json<RegisterRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    let email = body.email.trim().to_lowercase();
    if !valid_email(&email) {
        return Err(ApiError::bad_request("邮箱格式不正确"));
    }
    if !valid_password(&body.password) {
        return Err(ApiError::bad_request("密码长度至少 8 位"));
    }

    let conn = state.db.lock().unwrap();
    // 邮箱唯一性
    let exists: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM users WHERE email=?1)",
            rusqlite::params![email],
            |r| r.get(0),
        )
        .map_err(ApiError::from)?;
    if exists {
        return Err(ApiError::conflict("该邮箱已注册"));
    }

    let id = Uuid::new_v4().to_string();
    let now = now_ms();
    let password_hash = crate::auth::hash_password(&body.password)?;
    let display_name = body
        .display_name
        .clone()
        .unwrap_or_else(|| email.clone());

    conn.execute(
        "INSERT INTO users(id, email, password_hash, display_name, role, created_at) \
         VALUES(?1, ?2, ?3, ?4, 'member', ?5)",
        rusqlite::params![id, email, password_hash, display_name.clone(), now],
    )
    .map_err(ApiError::from)?;

    drop(conn); // 尽早释放锁

    let tokens = issue_tokens(&state.jwt, &id, "member", Some("free"));
    let user = json!({
        "id": id,
        "email": email,
        "display_name": display_name,
        "role": "member",
        "plan": "free",
        "created_at": now,
    });

    let req_id = crate::routes::next_request_id();
    Ok(Json(ApiResponse::with_data(json!({"user": user, "tokens": tokens}), &req_id)))
}

/// POST /api/v1/auth/login
pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    let email = body.email.trim().to_lowercase();
    let conn = state.db.lock().unwrap();

    let row: Option<(String, String, String, String, i64)> = conn
        .query_row(
            "SELECT id, password_hash, display_name, role, created_at FROM users WHERE email=?1",
            rusqlite::params![email],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
        )
        .optional()
        .map_err(ApiError::from)?;

    let Some((id, password_hash, display_name, role, created_at)) = row else {
        return Err(ApiError::unauthorized("邮箱或密码不正确"));
    };
    drop(conn);

    if !crate::auth::verify_password(&body.password, &password_hash) {
        return Err(ApiError::unauthorized("邮箱或密码不正确"));
    }

    // 读取订阅 plan
    let plan: String = state
        .db
        .lock()
        .unwrap()
        .query_row(
            "SELECT plan FROM subscriptions WHERE user_id=?1 AND status='active' \
             ORDER BY created_at DESC LIMIT 1",
            rusqlite::params![id],
            |r| r.get::<_, String>(0),
        )
        .optional()
        .map_err(ApiError::from)?
        .unwrap_or_else(|| "free".to_string());

    let tokens = issue_tokens(&state.jwt, &id, &role, Some(&plan));
    let user = json!({
        "id": id,
        "email": email,
        "display_name": display_name,
        "role": role,
        "plan": plan,
        "created_at": created_at,
    });

    let req_id = crate::routes::next_request_id();
    Ok(Json(ApiResponse::with_data(json!({"user": user, "tokens": tokens}), &req_id)))
}

/// POST /api/v1/auth/refresh
pub async fn refresh(
    State(state): State<AppState>,
    Json(body): Json<RefreshRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    let claims = validate_refresh(&state.jwt, &body.refresh_token)?;

    // 用户仍存在
    let exists: bool = state
        .db
        .lock()
        .unwrap()
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM users WHERE id=?1)",
            rusqlite::params![claims.sub],
            |r| r.get(0),
        )
        .map_err(ApiError::from)?;
    if !exists {
        return Err(ApiError::unauthorized("用户不存在"));
    }

    let tokens = issue_tokens(&state.jwt, &claims.sub, &claims.role, claims.plan.as_deref());
    let req_id = crate::routes::next_request_id();
    Ok(Json(ApiResponse::with_data(json!({"tokens": tokens}), &req_id)))
}

// 允许未使用 AuthUser 时通过（ME 路由使用，此处仅为类型检查兼容）
#[allow(dead_code)]
fn _auth_compat(_u: AuthUser) {}