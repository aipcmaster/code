//! 告警路由（ERD 3.14 alerts）。

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::{ApiResponse, AppState};
use axum::extract::State;
use axum::Json;
use serde_json::json;

/// GET /api/v1/alerts —— 当前用户设备的告警列表。
pub async fn list(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    let conn = state.db.lock().unwrap();
    let mut stmt = conn
        .prepare(
            "SELECT a.id, a.device_id, a.type, a.severity, a.title, a.message, a.status, a.created_at \
             FROM alerts a JOIN devices d ON a.device_id = d.id \
             WHERE d.user_id=?1 ORDER BY a.created_at DESC LIMIT 100",
        )
        .map_err(ApiError::from)?;
    let rows = stmt
        .query_map(rusqlite::params![auth.user_id], |r| {
            Ok(json!({
                "id": r.get::<_, String>(0)?,
                "device_id": r.get::<_, String>(1)?,
                "type": r.get::<_, String>(2)?,
                "severity": r.get::<_, String>(3)?,
                "title": r.get::<_, String>(4)?,
                "message": r.get::<_, Option<String>>(5)?,
                "status": r.get::<_, String>(6)?,
                "created_at": r.get::<_, i64>(7)?,
            }))
        })
        .map_err(ApiError::from)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(ApiError::from)?;

    let req_id = crate::routes::next_request_id();
    Ok(Json(ApiResponse::with_data(
        json!({"alerts": rows}),
        &req_id,
    )))
}
