//! 统一响应与错误模型（SD §4.1 / §4.4）。

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

/// 业务错误码（SD §4.4）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    Ok = 0,
    /// 参数错误
    BadRequest = 40001,
    /// 未认证
    Unauthorized = 40101,
    /// 无权限
    Forbidden = 40301,
    /// 资源不存在
    NotFound = 40401,
    /// 冲突
    Conflict = 40901,
    /// 请求过频
    TooManyRequests = 42901,
    /// 服务器错误
    Internal = 50001,
    /// 订阅过期
    SubscriptionExpired = 60001,
    /// 设备数量超限
    DeviceLimitExceeded = 60002,
}

/// 统一返回体（SD §4.1）。
#[derive(Debug, Clone, Serialize)]
pub struct ApiResponse<T = serde_json::Value> {
    pub code: i32,
    pub message: String,
    pub data: T,
    pub request_id: String,
}

impl ApiResponse<serde_json::Value> {
    /// 快速成功响应。
    pub fn ok(request_id: &str) -> Self {
        Self {
            code: ErrorCode::Ok as i32,
            message: "success".to_string(),
            data: serde_json::json!(null),
            request_id: request_id.to_string(),
        }
    }
}

impl<T: Serialize> ApiResponse<T> {
    pub fn with_data(data: T, request_id: &str) -> Self {
        Self {
            code: ErrorCode::Ok as i32,
            message: "success".to_string(),
            data,
            request_id: request_id.to_string(),
        }
    }
}

impl IntoResponse for ApiResponse<serde_json::Value> {
    fn into_response(self) -> Response {
        (StatusCode::OK, axum::Json(self)).into_response()
    }
}

/// API 错误：携带错误码与 HTTP 状态。
#[derive(Debug)]
pub struct ApiError {
    pub code: ErrorCode,
    pub message: String,
    pub status: StatusCode,
}

impl ApiError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        let status = match code {
            ErrorCode::BadRequest => StatusCode::BAD_REQUEST,
            ErrorCode::Unauthorized => StatusCode::UNAUTHORIZED,
            ErrorCode::Forbidden => StatusCode::FORBIDDEN,
            ErrorCode::NotFound => StatusCode::NOT_FOUND,
            ErrorCode::Conflict => StatusCode::CONFLICT,
            ErrorCode::TooManyRequests => StatusCode::TOO_MANY_REQUESTS,
            ErrorCode::SubscriptionExpired => StatusCode::PAYMENT_REQUIRED,
            ErrorCode::DeviceLimitExceeded => StatusCode::PAYMENT_REQUIRED,
            ErrorCode::Internal => StatusCode::INTERNAL_SERVER_ERROR,
            ErrorCode::Ok => StatusCode::OK,
        };
        Self {
            code,
            message: message.into(),
            status,
        }
    }

    pub fn bad_request(msg: impl Into<String>) -> Self {
        Self::new(ErrorCode::BadRequest, msg)
    }
    pub fn unauthorized(msg: impl Into<String>) -> Self {
        Self::new(ErrorCode::Unauthorized, msg)
    }
    pub fn forbidden(msg: impl Into<String>) -> Self {
        Self::new(ErrorCode::Forbidden, msg)
    }
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::new(ErrorCode::NotFound, msg)
    }
    pub fn conflict(msg: impl Into<String>) -> Self {
        Self::new(ErrorCode::Conflict, msg)
    }
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::new(ErrorCode::Internal, msg)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        // 用不可变字段
        let code = self.code as i32;
        let message = self.message.clone();
        let status = self.status;
        let body = ApiResponse {
            code,
            message,
            data: serde_json::json!(null),
            request_id: String::new(),
        };
        (status, axum::Json(body)).into_response()
    }
}

impl From<rusqlite::Error> for ApiError {
    fn from(e: rusqlite::Error) -> Self {
        tracing::error!("sqlite error: {e}");
        Self::internal("数据库错误")
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(e: serde_json::Error) -> Self {
        tracing::error!("json error: {e}");
        Self::internal("数据解析错误")
    }
}