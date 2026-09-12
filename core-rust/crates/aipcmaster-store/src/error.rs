//! 存储层错误。

/// 存储层错误。
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("SQLite 操作失败 [{context}]: {source}")]
    Sqlite {
        context: &'static str,
        #[source]
        source: rusqlite::Error,
    },
    #[error("I/O 失败 [{context}]: {source}")]
    Io {
        context: &'static str,
        #[source]
        source: std::io::Error,
    },
    #[error("JSON 序列化/反序列化失败: {0}")]
    Json(#[from] serde_json::Error),
    #[error("内部锁中毒")]
    LockPoisoned,
}

/// 存储层结果。
pub type Result<T> = std::result::Result<T, Error>;
