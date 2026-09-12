//! 统一错误类型。

use std::fmt;

/// 采集层错误。
#[derive(Debug)]
pub enum Error {
    /// I/O 失败，附上下文说明失败来源（如 "/proc/stat"）。
    Io {
        context: &'static str,
        source: std::io::Error,
    },
    /// 文件内容解析失败。
    Parse {
        context: &'static str,
        detail: String,
        line: Option<String>,
    },
    /// 当前平台尚未支持。
    UnsupportedPlatform(&'static str),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io { context, source } => write!(f, "采集失败 [{context}]: {source}"),
            Error::Parse {
                context,
                detail,
                line,
            } => {
                write!(f, "解析失败 [{context}]: {detail}")?;
                if let Some(l) = line {
                    write!(f, "; 行: {l:?}")?;
                }
                Ok(())
            }
            Error::UnsupportedPlatform(platform) => write!(f, "平台不受支持: {platform}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}

/// 采集层便捷结果类型。
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_messages() {
        let e = Error::UnsupportedPlatform("windows");
        assert!(e.to_string().contains("windows"));
    }
}
