//! 简易内存限流器（滑动窗口）—— 防登录暴力破解 / 批量注册。
//!
//! 说明：按「标识」（登录/注册用邮箱）计数，非按 IP。生产环境应在反向代理层
//! （Nginx / API 网关）叠加按 IP 限流，本模块作为应用层兜底。
//!
//! 线程安全：`Mutex<HashMap>`；键空间有上限，超过后清理过期键，避免内存无限增长。

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// 滑动窗口限流器。
pub struct RateLimiter {
    inner: Mutex<HashMap<String, Vec<Instant>>>,
    /// 窗口内允许的最大次数。
    max: usize,
    /// 时间窗口。
    window: Duration,
    /// 键空间上限（超出时清理过期键）。
    max_keys: usize,
}

impl RateLimiter {
    pub fn new(max: usize, window: Duration) -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
            max,
            window,
            max_keys: 10_000,
        }
    }

    /// 检查并记录一次请求。返回 `true` 表示允许，`false` 表示超限。
    pub fn check(&self, key: &str) -> bool {
        let now = Instant::now();
        let mut map = self.inner.lock().unwrap();

        // 键空间过大时清理过期键，防止内存膨胀
        if map.len() > self.max_keys {
            let window = self.window;
            map.retain(|_, stamps| {
                stamps.retain(|t| now.duration_since(*t) < window);
                !stamps.is_empty()
            });
        }

        let entry = map.entry(key.to_string()).or_default();
        entry.retain(|t| now.duration_since(*t) < self.window);
        if entry.len() >= self.max {
            return false;
        }
        entry.push(now);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allows_up_to_max_then_blocks() {
        let rl = RateLimiter::new(3, Duration::from_secs(60));
        assert!(rl.check("a"));
        assert!(rl.check("a"));
        assert!(rl.check("a"));
        assert!(!rl.check("a"), "超过上限应被拒绝");
    }

    #[test]
    fn separate_keys_independent() {
        let rl = RateLimiter::new(1, Duration::from_secs(60));
        assert!(rl.check("a"));
        assert!(rl.check("b"));
        assert!(!rl.check("a"));
    }

    #[test]
    fn window_expiry_allows_again() {
        let rl = RateLimiter::new(1, Duration::from_millis(30));
        assert!(rl.check("a"));
        assert!(!rl.check("a"));
        std::thread::sleep(Duration::from_millis(50));
        assert!(rl.check("a"), "窗口过期后应重新允许");
    }
}
