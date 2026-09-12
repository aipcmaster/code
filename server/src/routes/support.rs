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

/// POST /api/v1/support/tickets —— 创建工单。
pub async fn create_ticket(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<TicketRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    if body.subject.trim().is_empty() || body.content.trim().is_empty() {
        return Err(ApiError::bad_request("subject 与 content 必填"));
    }
    let id = Uuid::new_v4().to_string();
    let now = now_ms();
    let conn = state.db.lock().unwrap();
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
