//! 设备路由（ERD 3.4）。

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::{ApiResponse, AppState};
use axum::extract::{Path, State};
use axum::Json;
use serde::Deserialize;
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct RegisterDeviceRequest {
    pub device_name: String,
    pub os: Option<String>,
    pub version: Option<String>,
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}

/// GET /api/v1/devices —— 当前用户设备列表。
pub async fn list(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    let conn = state.db.lock().unwrap();
    let mut stmt = conn
        .prepare(
            "SELECT id, device_name, os, version, status, last_seen_at, created_at \
             FROM devices WHERE user_id=?1 ORDER BY created_at DESC",
        )
        .map_err(ApiError::from)?;
    let rows = stmt
        .query_map(rusqlite::params![auth.user_id], |r| {
            Ok(json!({
                "id": r.get::<_, String>(0)?,
                "device_name": r.get::<_, String>(1)?,
                "os": r.get::<_, Option<String>>(2)?,
                "version": r.get::<_, Option<String>>(3)?,
                "status": r.get::<_, String>(4)?,
                "last_seen_at": r.get::<_, Option<i64>>(5)?,
                "created_at": r.get::<_, i64>(6)?,
            }))
        })
        .map_err(ApiError::from)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(ApiError::from)?;

    let req_id = crate::routes::next_request_id();
    Ok(Json(ApiResponse::with_data(
        json!({"devices": rows}),
        &req_id,
    )))
}

/// 输入长度上限（防超大请求体导致的存储膨胀）。
const MAX_DEVICE_NAME_LEN: usize = 64;
const MAX_OS_LEN: usize = 64;
const MAX_VERSION_LEN: usize = 32;

/// POST /api/v1/devices/register —— 注册设备（免费版限 2 台，SD §4.4 60002）。
pub async fn register(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<RegisterDeviceRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    let name = body.device_name.trim();
    if name.is_empty() {
        return Err(ApiError::bad_request("device_name 必填"));
    }
    if name.len() > MAX_DEVICE_NAME_LEN {
        return Err(ApiError::bad_request("device_name 过长"));
    }
    if body.os.as_deref().is_some_and(|s| s.len() > MAX_OS_LEN) {
        return Err(ApiError::bad_request("os 过长"));
    }
    if body
        .version
        .as_deref()
        .is_some_and(|s| s.len() > MAX_VERSION_LEN)
    {
        return Err(ApiError::bad_request("version 过长"));
    }

    let conn = state.db.lock().unwrap();

    // 免费版设备数限制（当前统一 free 限制；pro 版不受限，此处简化）
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM devices WHERE user_id=?1 AND status='active'",
            rusqlite::params![auth.user_id],
            |r| r.get(0),
        )
        .map_err(ApiError::from)?;
    if count >= state.free_device_limit {
        return Err(ApiError::new(
            crate::error::ErrorCode::DeviceLimitExceeded,
            "免费版最多 2 台设备，升级 Pro 解锁更多",
        ));
    }

    let id = Uuid::new_v4().to_string();
    let now = now_ms();
    conn.execute(
        "INSERT INTO devices(id, user_id, device_name, os, version, status, last_seen_at, created_at) \
         VALUES(?1, ?2, ?3, ?4, ?5, 'active', ?6, ?7)",
        rusqlite::params![
            id,
            auth.user_id,
            body.device_name.trim(),
            body.os,
            body.version,
            now,
            now
        ],
    )
    .map_err(ApiError::from)?;

    let req_id = crate::routes::next_request_id();
    Ok(Json(ApiResponse::with_data(
        json!({"id": id, "status": "active"}),
        &req_id,
    )))
}

/// DELETE /api/v1/devices/{id} —— 解绑设备（软删除 status=inactive，保留审计痕迹）。
pub async fn unregister(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    let conn = state.db.lock().unwrap();
    let updated = conn
        .execute(
            "UPDATE devices SET status='inactive' WHERE id=?1 AND user_id=?2",
            rusqlite::params![id, auth.user_id],
        )
        .map_err(ApiError::from)?;
    if updated == 0 {
        return Err(ApiError::not_found("设备不存在或不属于当前用户"));
    }

    let req_id = crate::routes::next_request_id();
    Ok(Json(ApiResponse::with_data(
        json!({"id": id, "status": "inactive"}),
        &req_id,
    )))
}
