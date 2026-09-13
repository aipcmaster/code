//! API Key 路由（ERD 3.16 api_keys）。
//! 说明：创建时返回明文 key（仅此一次），数据库只存哈希（SD §6：API Key 哈希存储）。

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::{ApiResponse, AppState};
use axum::extract::State;
use axum::Json;
use base64::Engine;
use rand::RngCore;
use serde::Deserialize;
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreateApiKeyRequest {
    pub name: String,
    pub expires_at: Option<i64>,
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}

/// 生成随机 key 明文。
fn generate_key() -> String {
    let mut bytes = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    format!(
        "ak_{}",
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
    )
}

/// API Key 名称长度上限。
const MAX_NAME_LEN: usize = 64;

/// POST /api/v1/api-keys —— 创建 API Key（返回明文一次）。
pub async fn create(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<CreateApiKeyRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    if body.name.trim().is_empty() {
        return Err(ApiError::bad_request("name 必填"));
    }
    if body.name.len() > MAX_NAME_LEN {
        return Err(ApiError::bad_request("name 过长"));
    }
    let id = Uuid::new_v4().to_string();
    let now = now_ms();
    let plain = generate_key();

    // 哈希存储（SD §6）
    let key_hash = crate::auth::hash_password(&plain)?;

    let conn = state.db.lock().unwrap();
    conn.execute(
        "INSERT INTO api_keys(id, user_id, name, key_hash, scopes_json, last_used_at, expires_at, status, created_at) \
         VALUES(?1, ?2, ?3, ?4, '[\"read\",\"write\"]', NULL, ?5, 'active', ?6)",
        rusqlite::params![id, auth.user_id, body.name.trim(), key_hash, body.expires_at, now],
    )
    .map_err(ApiError::from)?;

    let req_id = crate::routes::next_request_id();
    Ok(Json(ApiResponse::with_data(
        json!({
            "id": id,
            "name": body.name,
            "api_key": plain, // 仅本次返回
            "expires_at": body.expires_at,
            "warning": "请立即保存，明文仅显示一次"
        }),
        &req_id,
    )))
}
