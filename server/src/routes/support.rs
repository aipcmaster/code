//! 支持工单路由（ERD 3.17 support_tickets）。

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::{ApiResponse, AppState};
use axum::extract::State;
use axum::Json;
use serde::Deserialize;
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct TicketRequest {
    pub subject: String,
    pub content: String,
    pub device_id: Option<String>,
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}

/// 输入长度上限（防超大请求体导致的存储膨胀）。
const MAX_SUBJECT_LEN: usize = 200;
const MAX_CONTENT_LEN: usize = 10_000;

/// POST /api/v1/support/tickets —— 创建工单。
pub async fn create_ticket(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<TicketRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    if body.subject.trim().is_empty() || body.content.trim().is_empty() {
        return Err(ApiError::bad_request("subject 与 content 必填"));
    }
    if body.subject.len() > MAX_SUBJECT_LEN {
        return Err(ApiError::bad_request("subject 过长"));
    }
    if body.content.len() > MAX_CONTENT_LEN {
        return Err(ApiError::bad_request("content 过长"));
    }

    let conn = state.db.lock().unwrap();

    // device_id 若提供，必须属于当前用户（防跨用户关联）
    if let Some(device_id) = body.device_id.as_deref() {
        let owned: bool = conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM devices WHERE id=?1 AND user_id=?2)",
                rusqlite::params![device_id, auth.user_id],
                |r| r.get(0),
            )
            .map_err(ApiError::from)?;
        if !owned {
            return Err(ApiError::bad_request("device_id 不存在或不属于当前用户"));
        }
    }

    let id = Uuid::new_v4().to_string();
    let now = now_ms();
    conn.execute(
        "INSERT INTO support_tickets(id, user_id, device_id, subject, content, status, created_at) \
         VALUES(?1, ?2, ?3, ?4, ?5, 'open', ?6)",
        rusqlite::params![
            id,
            auth.user_id,
            body.device_id,
            body.subject,
            body.content,
            now
        ],
    )
    .map_err(ApiError::from)?;

    let req_id = crate::routes::next_request_id();
    Ok(Json(ApiResponse::with_data(
        json!({"id": id, "status": "open", "created_at": now}),
        &req_id,
    )))
}
