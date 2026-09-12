//! 服务端集成测试：通过真实 HTTP 层（tower oneshot）验证核心 API 链路。

use aipcmaster_server::{app, AppState};
use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use tower::ServiceExt;

/// 发送 JSON 请求并解析响应体。
async fn send_json(
    app: &axum::Router,
    method: &str,
    path: &str,
    token: Option<&str>,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let mut builder = Request::builder().method(method).uri(path);
    if let Some(t) = token {
        builder = builder.header("Authorization", format!("Bearer {t}"));
    }
    let req = match body {
        Some(b) => builder
            .header("Content-Type", "application/json")
            .body(Body::from(b.to_string()))
            .unwrap(),
        None => builder.body(Body::empty()).unwrap(),
    };
    let res = app.clone().oneshot(req).await.unwrap();
    let status = res.status();
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    let json = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, json)
}

/// 注册并返回 access token。
async fn register_and_token(app: &axum::Router, email: &str) -> String {
    let (status, body) = send_json(
        app,
        "POST",
        "/api/v1/auth/register",
        None,
        Some(json!({"email": email, "password": "password123", "display_name": "测试用户"})),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "register 失败: {body}");
    body["data"]["tokens"]["access_token"]
        .as_str()
        .unwrap()
        .to_string()
}

#[tokio::test]
async fn auth_register_login_refresh_flow() {
    let state = AppState::in_memory().unwrap();
    let app = app(state);

    // 注册
    let token = register_and_token(&app, "user1@example.com").await;
    assert!(!token.is_empty());

    // 重复注册 → 40901
    let (status, body) = send_json(
        &app,
        "POST",
        "/api/v1/auth/register",
        None,
        Some(json!({"email": "user1@example.com", "password": "password123"})),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["code"], 40901);

    // 登录成功
    let (status, body) = send_json(
        &app,
        "POST",
        "/api/v1/auth/login",
        None,
        Some(json!({"email": "user1@example.com", "password": "password123"})),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "login 失败: {body}");
    let login_token = body["data"]["tokens"]["access_token"].as_str().unwrap();
    let refresh_token = body["data"]["tokens"]["refresh_token"].as_str().unwrap();

    // 刷新
    let (status, body) = send_json(
        &app,
        "POST",
        "/api/v1/auth/refresh",
        None,
        Some(json!({"refresh_token": refresh_token})),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "refresh 失败: {body}");
    assert!(body["data"]["tokens"]["access_token"].is_string());

    // 登录失败（错密码）→ 40101
    let (status, body) = send_json(
        &app,
        "POST",
        "/api/v1/auth/login",
        None,
        Some(json!({"email": "user1@example.com", "password": "wrongpass"})),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["code"], 40101);

    let _ = login_token;
}

#[tokio::test]
async fn me_requires_auth() {
    let state = AppState::in_memory().unwrap();
    let app = app(state);

    // 无 token → 40101
    let (status, body) = send_json(&app, "GET", "/api/v1/users/me", None, None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["code"], 40101);

    // 带 token 正常
    let token = register_and_token(&app, "me@example.com").await;
    let (status, body) = send_json(&app, "GET", "/api/v1/users/me", Some(&token), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["email"], "me@example.com");
    assert_eq!(body["data"]["role"], "member");
    assert_eq!(body["data"]["plan"], "free");
    assert!(body["request_id"].as_str().unwrap().starts_with("req_"));
}

#[tokio::test]
async fn device_lifecycle_and_limit() {
    let state = AppState::in_memory().unwrap();
    let app = app(state);
    let token = register_and_token(&app, "dev@example.com").await;

    // 空列表
    let (status, body) = send_json(&app, "GET", "/api/v1/devices", Some(&token), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["devices"].as_array().unwrap().len(), 0);

    // 注册设备（免费版限 2 台）
    let mut ids = vec![];
    for n in 1..=2 {
        let (status, body) = send_json(
            &app,
            "POST",
            "/api/v1/devices/register",
            Some(&token),
            Some(json!({"device_name": format!("PC-{n}"), "os": "Linux"})),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "设备 {n} 注册失败: {body}");
        ids.push(body["data"]["id"].as_str().unwrap().to_string());
    }

    // 第三台 → 60002 超限
    let (status, body) = send_json(
        &app,
        "POST",
        "/api/v1/devices/register",
        Some(&token),
        Some(json!({"device_name": "PC-3", "os": "Linux"})),
    )
    .await;
    assert_eq!(status, StatusCode::PAYMENT_REQUIRED);
    assert_eq!(body["code"], 60002);

    // 解绑一台后释放额度
    let (status, _) = send_json(
        &app,
        "DELETE",
        &format!("/api/v1/devices/{}", ids[0]),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (status, body) = send_json(
        &app,
        "POST",
        "/api/v1/devices/register",
        Some(&token),
        Some(json!({"device_name": "PC-3", "os": "Linux"})),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "解绑后应可再注册: {body}");

    // 解绑不存在的设备 → 40401
    let (status, body) = send_json(
        &app,
        "DELETE",
        "/api/v1/devices/not-exist-id",
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["code"], 40401);
}

#[tokio::test]
async fn diagnostic_session_and_report() {
    let state = AppState::in_memory().unwrap();
    let app = app(state);
    let token = register_and_token(&app, "diag@example.com").await;

    // 注册设备
    let (status, body) = send_json(
        &app,
        "POST",
        "/api/v1/devices/register",
        Some(&token),
        Some(json!({"device_name": "PC-DIAG", "os": "Linux"})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let device_id = body["data"]["id"].as_str().unwrap().to_string();

    // 创建设备不属于自己的会话 → 40401
    let (status, body) = send_json(
        &app,
        "POST",
        "/api/v1/diagnostics/sessions",
        Some(&token),
        Some(json!({"device_id": "other-device"})),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["code"], 40401);

    // 正常创建会话
    let (status, body) = send_json(
        &app,
        "POST",
        "/api/v1/diagnostics/sessions",
        Some(&token),
        Some(json!({"device_id": device_id, "trigger_type": "manual"})),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "创建诊断失败: {body}");
    assert_eq!(body["data"]["status"], "running");
    assert!(body["data"]["id"].is_string());
}

#[tokio::test]
async fn optimization_state_machine() {
    let state = AppState::in_memory().unwrap();
    let app = app(state);
    let token = register_and_token(&app, "opt@example.com").await;

    // 注册设备
    let (_, body) = send_json(
        &app,
        "POST",
        "/api/v1/devices/register",
        Some(&token),
        Some(json!({"device_name": "PC-OPT", "os": "Linux"})),
    )
    .await;
    let device_id = body["data"]["id"].as_str().unwrap().to_string();

    // 1. 提议优化动作（low risk，无需确认）
    let (status, body) = send_json(
        &app,
        "POST",
        "/api/v1/optimizations/actions",
        Some(&token),
        Some(json!({
            "device_id": device_id,
            "action_type": "clean_memory",
            "risk_level": "low",
            "requires_confirm": false,
            "parameters": {"target": "page_cache"}
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "提议失败: {body}");
    let action_id = body["data"]["id"].as_str().unwrap().to_string();

    // 2. 执行（低风险无需确认）
    let (status, body) = send_json(
        &app,
        "POST",
        &format!("/api/v1/optimizations/actions/{action_id}/execute"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "执行失败: {body}");
    let log_id = body["data"]["log_id"].as_str().unwrap().to_string();
    assert_eq!(body["data"]["status"], "executed");

    // 3. 重复执行 → 40901 冲突
    let (status, body) = send_json(
        &app,
        "POST",
        &format!("/api/v1/optimizations/actions/{action_id}/execute"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["code"], 40901);

    // 4. 回滚
    let (status, body) = send_json(
        &app,
        "POST",
        &format!("/api/v1/optimizations/logs/{log_id}/rollback"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "回滚失败: {body}");
    assert_eq!(body["data"]["rollback_status"], "success");
}

#[tokio::test]
async fn high_risk_action_requires_confirm() {
    let state = AppState::in_memory().unwrap();
    let app = app(state);
    let token = register_and_token(&app, "risky@example.com").await;

    let (_, body) = send_json(
        &app,
        "POST",
        "/api/v1/devices/register",
        Some(&token),
        Some(json!({"device_name": "PC-RISKY", "os": "Linux"})),
    )
    .await;
    let device_id = body["data"]["id"].as_str().unwrap().to_string();

    // 高危动作 requires_confirm=true 直接执行 → 拒绝
    let (status, body) = send_json(
        &app,
        "POST",
        "/api/v1/optimizations/actions",
        Some(&token),
        Some(json!({
            "device_id": device_id,
            "action_type": "disk",
            "risk_level": "high",
            "requires_confirm": true,
            "parameters": {}
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let action_id = body["data"]["id"].as_str().unwrap().to_string();

    let (status, body) = send_json(
        &app,
        "POST",
        &format!("/api/v1/optimizations/actions/{action_id}/execute"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["code"], 40001);
}

#[tokio::test]
async fn api_key_and_ticket_and_subscription() {
    let state = AppState::in_memory().unwrap();
    let app = app(state.clone());
    let token = register_and_token(&app, "extra@example.com").await;

    // API Key
    let (status, body) = send_json(
        &app,
        "POST",
        "/api/v1/api-keys",
        Some(&token),
        Some(json!({"name": "ci-key"})),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "api-key 创建失败: {body}");
    assert!(body["data"]["api_key"].as_str().unwrap().starts_with("ak_"));
    // 明文不落库：库中应只有哈希
    let stored: String = state
        .db
        .lock()
        .unwrap()
        .query_row("SELECT key_hash FROM api_keys", [], |r| r.get(0))
        .unwrap();
    assert_ne!(stored, body["data"]["api_key"].as_str().unwrap());

    // 订阅：free → checkout pro → current active
    let (status, body) = send_json(
        &app,
        "GET",
        "/api/v1/subscriptions/current",
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["plan"], "free");

    let (status, body) = send_json(
        &app,
        "POST",
        "/api/v1/subscriptions/checkout",
        Some(&token),
        Some(json!({"plan": "pro"})),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "checkout 失败: {body}");
    assert_eq!(body["data"]["plan"], "pro");

    let (status, body) = send_json(
        &app,
        "GET",
        "/api/v1/subscriptions/current",
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["plan"], "pro");
    assert_eq!(body["data"]["status"], "active");

    // 工单
    let (status, body) = send_json(
        &app,
        "POST",
        "/api/v1/support/tickets",
        Some(&token),
        Some(json!({"subject": "问题", "content": "求助"})),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "工单创建失败: {body}");
    assert_eq!(body["data"]["status"], "open");

    // 空订阅工单 → 40001
    let (status, body) = send_json(
        &app,
        "POST",
        "/api/v1/support/tickets",
        Some(&token),
        Some(json!({"subject": "", "content": ""})),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["code"], 40001);
}
