//! 诊断路由（ERD 3.9 / 3.10）。

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::{ApiResponse, AppState};
use axum::extract::{Path, State};
use axum::Json;
use rusqlite::OptionalExtension;
use serde::Deserialize;
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreateSessionRequest {
    pub device_id: String,
    pub trigger_type: Option<String>, // manual / scheduled / anomaly
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}

/// POST /api/v1/diagnostics/sessions —— 创建诊断会话，返回会话 ID。
/// 客户端随后上报脱敏指标，云端保存摘要（SD §5：云端仅存脱敏摘要）。
pub async fn create_session(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<CreateSessionRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    let conn = state.db.lock().unwrap();

    // 校验设备归属
    let owned: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM devices WHERE id=?1 AND user_id=?2)",
            rusqlite::params![body.device_id, auth.user_id],
            |r| r.get(0),
        )
        .map_err(ApiError::from)?;
    if !owned {
        return Err(ApiError::not_found("设备不存在或不属于当前用户"));
    }

    let id = Uuid::new_v4().to_string();
    let now = now_ms();
    let trigger = body.trigger_type.unwrap_or_else(|| "manual".to_string());
    // 白名单校验（防任意值写入）
    if !matches!(trigger.as_str(), "manual" | "scheduled" | "anomaly") {
        return Err(ApiError::bad_request("trigger_type 非法"));
    }
    conn.execute(
        "INSERT INTO diagnostic_sessions(id, user_id, device_id, trigger_type, status, created_at) \
         VALUES(?1, ?2, ?3, ?4, 'running', ?5)",
        rusqlite::params![id, auth.user_id, body.device_id, trigger, now],
    )
    .map_err(ApiError::from)?;

    let req_id = crate::routes::next_request_id();
    Ok(Json(ApiResponse::with_data(
        json!({
            "id": id,
            "status": "running",
            "trigger_type": trigger,
            "created_at": now,
            "message": "请客户端采集快照并回传摘要"
        }),
        &req_id,
    )))
}

/// GET /api/v1/diagnostics/reports/{id} —— 获取诊断报告（脱敏摘要）。
pub async fn get_report(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    let conn = state.db.lock().unwrap();
    let row: Option<(String, String, i64, String, Option<String>, i64)> = conn
        .query_row(
            "SELECT r.id, r.device_id, r.health_score, r.summary, r.issues_json, r.created_at \
             FROM diagnostic_reports r \
             JOIN diagnostic_sessions s ON r.session_id = s.id \
             WHERE r.id=?1 AND s.user_id=?2",
            rusqlite::params![id, auth.user_id],
            |r| {
                Ok((
                    r.get(0)?,
                    r.get(1)?,
                    r.get(2)?,
                    r.get(3)?,
                    r.get(4)?,
                    r.get(5)?,
                ))
            },
        )
        .optional()
        .map_err(ApiError::from)?;

    let Some((id, device_id, score, summary, issues, created_at)) = row else {
        return Err(ApiError::not_found("报告不存在"));
    };

    let req_id = crate::routes::next_request_id();
    Ok(Json(ApiResponse::with_data(
        json!({
            "id": id,
            "device_id": device_id,
            "health_score": score,
            "summary": summary,
            "issues": issues
                .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok()),
            "created_at": created_at
        }),
        &req_id,
    )))
}
