//! 优化路由（ERD 3.11 optimization_actions / 3.12 optimization_logs）。
//! 职责：创建优化动作建议 → 执行（记录日志）→ 回滚（补记 rollback 状态）。
//!
//! 注意：真正的系统变更由**客户端引擎**执行（决策执行层，默认只读）；
//! 云端只记录动作与结果状态机（suggested/approved/executed/rolled_back）。

use crate::auth::AuthUser;
use crate::error::ApiError;
use rusqlite::OptionalExtension;
use crate::{AppState, ApiResponse};
use axum::extract::{Path, State};
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreateActionRequest {
    pub device_id: String,
    pub issue_id: Option<String>,
    pub action_type: String, // clean_memory / startup / disk
    pub risk_level: String,  // low / medium / high
    pub requires_confirm: bool,
    pub parameters: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct ActionView {
    pub id: String,
    pub device_id: String,
    pub action_type: String,
    pub risk_level: String,
    pub requires_confirm: bool,
    pub status: String,
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}

/// 创建优化动作建议（客户端诊断后调用）。
pub async fn propose(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<CreateActionRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    let conn = state.db.lock().unwrap();
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
    let params = body.parameters.map(|p| p.to_string());
    conn.execute(
        "INSERT INTO optimization_actions(id, issue_id, device_id, action_type, risk_level, requires_confirm, parameters_json, status, created_at) \
         VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, 'suggested', ?8)",
        rusqlite::params![
            id,
            body.issue_id,
            body.device_id,
            body.action_type,
            body.risk_level,
            body.requires_confirm as i64,
            params,
            now
        ],
    )
    .map_err(ApiError::from)?;

    let req_id = crate::routes::next_request_id();
    Ok(Json(ApiResponse::with_data(
        json!({"id": id, "status": "suggested"}),
        &req_id,
    )))
}

/// POST /api/v1/optimizations/actions/{id}/execute —— 执行优化（状态机 suggested/approved → executed）。
pub async fn execute(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    let conn = state.db.lock().unwrap();

    // 拉取动作（校验归属）
    let row: Option<(String, String, String, String, i64)> = conn
        .query_row(
            "SELECT a.device_id, a.action_type, a.status, a.risk_level, a.requires_confirm \
             FROM optimization_actions a JOIN devices d ON a.device_id = d.id \
             WHERE a.id=?1 AND d.user_id=?2",
            rusqlite::params![id, auth.user_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
        )
        .optional()
        .map_err(ApiError::from)?;

    let Some((device_id, action_type, status, _risk, requires_confirm)) = row else {
        return Err(ApiError::not_found("优化动作不存在"));
    };
    if status != "suggested" && status != "approved" {
        return Err(ApiError::conflict(format!("动作状态为 {status}，不可执行")));
    }
    // 高危动作要求先显式确认（调用方提交 confirm=true；此处简化：requires_confirm 必须为 false 才直接执行）
    if requires_confirm == 1 {
        return Err(ApiError::bad_request("此动作需要先在客户端确认（requires_confirm=true）"));
    }

    // 执行（云端仅记录；实际系统变更在客户端引擎）
    let action_result = "success";
    let now = now_ms();
    let log_id = Uuid::new_v4().to_string();
    conn.execute(
        "UPDATE optimization_actions SET status='executed' WHERE id=?1",
        rusqlite::params![id],
    )
    .map_err(ApiError::from)?;
    conn.execute(
        "INSERT INTO optimization_logs(id, action_id, device_id, user_id, executed_at, result, rollback_status, log_text) \
         VALUES(?1, ?2, ?3, ?4, ?5, ?6, 'none', ?7)",
        rusqlite::params![
            log_id,
            id,
            device_id,
            auth.user_id,
            now,
            action_result,
            format!("action {} executed on {}", action_type, device_id)
        ],
    )
    .map_err(ApiError::from)?;

    let req_id = crate::routes::next_request_id();
    Ok(Json(ApiResponse::with_data(
        json!({"action_id": id, "log_id": log_id, "status": "executed", "result": action_result}),
        &req_id,
    )))
}

/// POST /api/v1/optimizations/logs/{id}/rollback —— 回滚（状态机 executed → rolled_back）。
pub async fn rollback(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(log_id): Path<String>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    let conn = state.db.lock().unwrap();
    let row: Option<(String, String)> = conn
        .query_row(
            "SELECT l.action_id, l.result FROM optimization_logs l \
             JOIN optimization_actions a ON l.action_id = a.id \
             JOIN devices d ON a.device_id = d.id \
             WHERE l.id=?1 AND d.user_id=?2",
            rusqlite::params![log_id, auth.user_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()
        .map_err(ApiError::from)?;
    let Some((action_id, _)) = row else {
        return Err(ApiError::not_found("执行日志不存在"));
    };

    // 回滚（云端记录状态；实际回滚在客户端引擎执行）
    conn.execute(
        "UPDATE optimization_logs SET rollback_status='success' WHERE id=?1",
        rusqlite::params![log_id],
    )
    .map_err(ApiError::from)?;
    conn.execute(
        "UPDATE optimization_actions SET status='rolled_back' WHERE id=?1",
        rusqlite::params![action_id],
    )
    .map_err(ApiError::from)?;

    let req_id = crate::routes::next_request_id();
    Ok(Json(ApiResponse::with_data(
        json!({"log_id": log_id, "action_id": action_id, "rollback_status": "success"}),
        &req_id,
    )))
}

#[allow(dead_code)]
fn _unused(_: ActionView) {}