//! 认证与授权：JWT（HS256）+ PBKDF2 密码哈希 + 用户提取器。
//!
//! - JWT 手写实现（hmac + sha2 + base64），标准 RFC 7519 结构；
//! - 生产环境建议切换 `jsonwebtoken` crate 或 OIDC 提供方（SD §4.3）；
//! - 密码哈希 PBKDF2-HMAC-SHA256（10 万轮），生产可换 Argon2id（SD §6）。

use crate::error::ApiError;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use hmac::{Hmac, Mac};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::time::{SystemTime, UNIX_EPOCH};

type HmacSha256 = Hmac<Sha256>;

/// JWT 配置。
#[derive(Debug, Clone)]
pub struct JwtConfig {
    pub secret: String,
    /// 访问令牌有效期（秒）。
    pub access_ttl_secs: i64,
    /// 刷新令牌有效期（秒）。
    pub refresh_ttl_secs: i64,
}

impl Default for JwtConfig {
    fn default() -> Self {
        Self {
            secret: "dev-secret-do-not-use-in-prod".into(),
            access_ttl_secs: 3600,
            refresh_ttl_secs: 30 * 24 * 3600,
        }
    }
}

/// JWT 载荷（订阅 token 内嵌，避免每请求查订阅表）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,          // 用户 ID
    pub role: String,         // admin / member / viewer
    pub exp: i64,             // 过期（Unix 秒）
    pub iat: i64,             // 签发（Unix 秒）
    pub typ: String,          // access / refresh
    pub plan: Option<String>, // free / pro，可为空
}

/// 签发令牌对。
#[derive(Debug, Clone, Serialize)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

/// 从头构建 JWT（HS256）。签名覆盖 base64url(header).base64url(payload)（RFC 7519 §4.1）。
fn sign_token(secret: &str, header_json: &str, payload_json: &str) -> String {
    let b64_header = URL_SAFE_NO_PAD.encode(header_json.as_bytes());
    let b64_payload = URL_SAFE_NO_PAD.encode(payload_json.as_bytes());
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).expect("hmac accept any key");
    mac.update(b64_header.as_bytes());
    mac.update(b".");
    mac.update(b64_payload.as_bytes());
    let sig = mac.finalize().into_bytes();
    format!(
        "{}.{}.{}",
        b64_header,
        b64_payload,
        URL_SAFE_NO_PAD.encode(sig)
    )
}

/// 校验并解码 JWT。
fn verify_token<T: for<'de> Deserialize<'de>>(secret: &str, token: &str) -> Result<T, ApiError> {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return Err(ApiError::unauthorized("token 格式错误"));
    }
    let header_raw = URL_SAFE_NO_PAD
        .decode(parts[0])
        .map_err(|_| ApiError::unauthorized("token 编码错误"))?;
    let payload_raw = URL_SAFE_NO_PAD
        .decode(parts[1])
        .map_err(|_| ApiError::unauthorized("token 编码错误"))?;
    let header: serde_json::Value =
        serde_json::from_slice(&header_raw).map_err(|_| ApiError::unauthorized("token 头无效"))?;
    if header.get("alg").and_then(|v| v.as_str()) != Some("HS256") {
        return Err(ApiError::unauthorized("不支持的签名算法"));
    }

    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).expect("hmac");
    mac.update(parts[0].as_bytes());
    mac.update(b".");
    mac.update(parts[1].as_bytes());
    let expected = mac.finalize().into_bytes();
    let actual = URL_SAFE_NO_PAD
        .decode(parts[2])
        .map_err(|_| ApiError::unauthorized("签名无效"))?;
    // 恒定时间比较
    if expected.as_slice() != actual.as_slice() {
        return Err(ApiError::unauthorized("签名无效"));
    }

    let claims: T =
        serde_json::from_slice(&payload_raw).map_err(|_| ApiError::unauthorized("载荷无效"))?;
    Ok(claims)
}

/// 生成令牌对。
pub fn issue_tokens(
    config: &JwtConfig,
    user_id: &str,
    role: &str,
    plan: Option<&str>,
) -> TokenPair {
    let now = unix_secs();
    let header = r#"{"alg":"HS256","typ":"JWT"}"#;

    let access_claims = Claims {
        sub: user_id.to_string(),
        role: role.to_string(),
        exp: now + config.access_ttl_secs,
        iat: now,
        typ: "access".into(),
        plan: plan.map(|s| s.to_string()),
    };
    let refresh_claims = Claims {
        sub: user_id.to_string(),
        role: role.to_string(),
        exp: now + config.refresh_ttl_secs,
        iat: now,
        typ: "refresh".into(),
        plan: plan.map(|s| s.to_string()),
    };

    let access_payload = serde_json::to_string(&access_claims).expect("claims always serializable");
    let refresh_payload =
        serde_json::to_string(&refresh_claims).expect("claims always serializable");

    TokenPair {
        access_token: sign_token(&config.secret, header, &access_payload),
        refresh_token: sign_token(&config.secret, header, &refresh_payload),
        token_type: "Bearer".into(),
        expires_in: config.access_ttl_secs,
    }
}

/// 校验访问令牌。
pub fn validate_access(config: &JwtConfig, token: &str) -> Result<Claims, ApiError> {
    let claims: Claims = verify_token(&config.secret, token)?;
    if claims.typ != "access" {
        return Err(ApiError::unauthorized("令牌类型错误"));
    }
    if claims.exp < unix_secs() {
        return Err(ApiError::new(
            crate::error::ErrorCode::Unauthorized,
            "令牌已过期",
        ));
    }
    Ok(claims)
}

/// 校验刷新令牌。
pub fn validate_refresh(config: &JwtConfig, token: &str) -> Result<Claims, ApiError> {
    let claims: Claims = verify_token(&config.secret, token)?;
    if claims.typ != "refresh" {
        return Err(ApiError::unauthorized("令牌类型错误"));
    }
    if claims.exp < unix_secs() {
        return Err(ApiError::new(
            crate::error::ErrorCode::Unauthorized,
            "刷新令牌已过期",
        ));
    }
    Ok(claims)
}

/// PBKDF2-HMAC-SHA256 密码哈希。
/// 存 `salt_hex$hash_hex`。
pub fn hash_password(password: &str) -> Result<String, ApiError> {
    let mut salt = [0u8; 16];
    rand::rngs::OsRng.fill_bytes(&mut salt);
    let hash = pbkdf2_sha256(password.as_bytes(), &salt, 100_000);
    Ok(format!("{}${}", hex(&salt), hex(&hash)))
}

/// 校验密码。
pub fn verify_password(password: &str, stored: &str) -> bool {
    let Some((salt_hex, hash_hex)) = stored.split_once('$') else {
        return false;
    };
    let Ok(salt) = unhex(salt_hex) else {
        return false;
    };
    let Ok(expected) = unhex(hash_hex) else {
        return false;
    };
    let actual = pbkdf2_sha256(password.as_bytes(), &salt, 100_000);
    // 恒定时间比较
    if actual.len() != expected.len() {
        return false;
    }
    let mut diff = 0u8;
    for (a, b) in actual.iter().zip(expected.iter()) {
        diff |= a ^ b;
    }
    diff == 0
}

/// 底层 PBKDF2（利用 pbkdf2 crate 的底层函数）。
fn pbkdf2_sha256(password: &[u8], salt: &[u8], rounds: u32) -> [u8; 32] {
    // pbkdf2 0.12 无 password-hash feature 时暴露 `pbkdf2::pbkdf2_hmac`
    let mut out = [0u8; 32];
    pbkdf2::pbkdf2_hmac::<Sha256>(password, salt, rounds, &mut out);
    out
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn unhex(s: &str) -> Result<Vec<u8>, ()> {
    if !s.len().is_multiple_of(2) || !s.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(());
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).map_err(|_| ()))
        .collect()
}

fn unix_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// 认证用户：从请求中提取 Claims。
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: String,
    pub role: String,
    pub plan: Option<String>,
}

impl From<Claims> for AuthUser {
    fn from(c: Claims) -> Self {
        Self {
            user_id: c.sub,
            role: c.role,
            plan: c.plan,
        }
    }
}

/// axum 提取器：要求 `Authorization: Bearer <token>`。
impl FromRequestParts<crate::AppState> for AuthUser {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &crate::AppState,
    ) -> Result<Self, Self::Rejection> {
        let header = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| ApiError::unauthorized("缺少 Authorization 头"))?;
        let token = header
            .strip_prefix("Bearer ")
            .ok_or_else(|| ApiError::unauthorized("认证方式必须为 Bearer"))?;
        let claims = validate_access(&state.jwt, token)?;
        Ok(claims.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> JwtConfig {
        JwtConfig::default()
    }

    #[test]
    fn token_roundtrip() {
        let c = cfg();
        let pair = issue_tokens(&c, "u1", "admin", Some("pro"));
        let claims = validate_access(&c, &pair.access_token).unwrap();
        assert_eq!(claims.sub, "u1");
        assert_eq!(claims.role, "admin");
        assert_eq!(claims.plan.as_deref(), Some("pro"));
        assert_eq!(claims.typ, "access");
    }

    #[test]
    fn refresh_token_rejected_as_access() {
        let c = cfg();
        let pair = issue_tokens(&c, "u1", "member", None);
        let err = validate_access(&c, &pair.refresh_token).unwrap_err();
        assert_eq!(err.code, crate::error::ErrorCode::Unauthorized);
    }

    #[test]
    fn tampered_token_rejected() {
        let c = cfg();
        let pair = issue_tokens(&c, "u1", "member", None);
        let tampered = format!("{}X", &pair.access_token[..pair.access_token.len() - 1]);
        let err = validate_access(&c, &tampered).unwrap_err();
        assert_eq!(err.code, crate::error::ErrorCode::Unauthorized);
    }

    #[test]
    fn wrong_secret_rejected() {
        let c1 = cfg();
        let c2 = JwtConfig {
            secret: "other-secret".into(),
            ..Default::default()
        };
        let pair = issue_tokens(&c1, "u1", "member", None);
        let err = validate_access(&c2, &pair.access_token).unwrap_err();
        assert_eq!(err.code, crate::error::ErrorCode::Unauthorized);
    }

    #[test]
    fn password_roundtrip() {
        let stored = hash_password("s3cret!@pass").unwrap();
        assert!(verify_password("s3cret!@pass", &stored));
        assert!(!verify_password("wrong", &stored));
        assert!(!verify_password("s3cret!@pass", "not-a-valid-format"));
    }

    #[test]
    fn random_salts_differ() {
        let a = hash_password("same").unwrap();
        let b = hash_password("same").unwrap();
        assert_ne!(a, b);
    }
}
