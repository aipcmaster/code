//! 审计日志路由（ERD 3.15 audit_logs，SD §6 审计合规）。

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::{AppState, ApiResponse};
use axum::extract::State;
use axum::Json;
use serde_json::json;

/// GET /api/v1/audit-logs —— 当前用户的审计日志。
pub async fn list(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    let conn = state.db.lock().unwrap();
    let mut stmt = conn
        .prepare(
            "SELECT id, action, resource_type, resource_id, ip, user_agent, created_at \
             FROM audit_logs WHERE actor_user_id=?1 ORDER BY created_at DESC LIMIT 200",
        )
        .map_err(ApiError::from)?;
    let rows = stmt
        .query_map(rusqlite::params![auth.user_id], |r| {
            Ok(json!({
                "id": r.get::<_, String>(0)?,
                "action": r.get::<_, String>(1)?,
                "resource_type": r.get::<_, String>(2)?,
                "resource_id": r.get::<_, Option<String>>(3)?,
                "ip": r.get::<_, Option<String>>(4)?,
                "user_agent": r.get::<_, Option<String>>(5)?,
                "created_at": r.get::<_, i64>(6)?,
            }))
        })
        .map_err(ApiError::from)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(ApiError::from)?;

    let req_id = crate::routes::next_request_id();
    Ok(Json(ApiResponse::with_data(json!({"audit_logs": rows}), &req_id)))
}