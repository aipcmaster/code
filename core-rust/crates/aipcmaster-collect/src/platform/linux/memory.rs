//! 内存采集：`/proc/meminfo`。

use crate::metrics::MemoryMetrics;

/// 解析 `/proc/meminfo` 文本，返回 (MemTotal, MemAvailable, SwapTotal, SwapFree)，单位字节。
/// 输入是 kB，全部乘 1024。
pub(crate) fn parse_meminfo(text: &str) -> Option<(u64, u64, u64, u64)> {
    let mut total = None;
    let mut avail = None;
    let mut swap_total = None;
    let mut swap_free = None;

    for line in text.lines() {
        let mut parts = line.splitn(2, ':');
        let (Some(key), Some(rest)) = (parts.next(), parts.next()) else {
            continue;
        };
        let key = key.trim();
        let val_kb: u64 = rest
            .split_whitespace()
            .next()
            .and_then(|v| v.parse().ok())?;
        match key {
            "MemTotal" => total = Some(val_kb),
            "MemAvailable" => avail = Some(val_kb),
            "SwapTotal" => swap_total = Some(val_kb),
            "SwapFree" => swap_free = Some(val_kb),
            _ => {}
        }
    }

    let total = total?;
    let avail = avail.unwrap_or(total); // 旧内核无 MemAvailable -> 保守地视为可用=总量
    let swap_total = swap_total.unwrap_or(0);
    let swap_free = swap_free.unwrap_or(0);

    Some((
        total * 1024,
        avail * 1024,
        swap_total * 1024,
        swap_free * 1024,
    ))
}

/// 读取并计算当前内存指标。
pub(crate) fn sample() -> Option<MemoryMetrics> {
    let text = std::fs::read_to_string("/proc/meminfo").ok()?;
    let (total, avail, swap_total, swap_free) = parse_meminfo(&text)?;

    let used = total.saturating_sub(avail);
    let usage_percent = if total == 0 {
        0.0
    } else {
        (used as f64) * 100.0 / (total as f64)
    };

    Some(MemoryMetrics {
        total_bytes: total,
        used_bytes: used,
        available_bytes: avail,
        usage_percent,
        swap_total_bytes: swap_total,
        swap_used_bytes: swap_total.saturating_sub(swap_free),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const MEMINFO: &str = "\
MemTotal:       16384000 kB
MemFree:         2000000 kB
MemAvailable:    8000000 kB
Buffers:          100000 kB
Cached:          4000000 kB
SwapCached:            0 kB
SwapTotal:       4194304 kB
SwapFree:        4000000 kB
--- 其他行应忽略
";

    #[test]
    fn parses_meminfo() {
        let (total, avail, swap_total, swap_free) = parse_meminfo(MEMINFO).unwrap();
        assert_eq!(total, 16384000 * 1024);
        assert_eq!(avail, 8000000 * 1024);
        assert_eq!(swap_total, 4194304 * 1024);
        assert_eq!(swap_free, 4000000 * 1024);
    }

    #[test]
    fn sample_builds_metrics() {
        // 经 parse_meminfo + sample 逻辑（不触碰真实 /proc）
        let (total, avail, swap_total, swap_free) = parse_meminfo(MEMINFO).unwrap();
        let used = total.saturating_sub(avail);
        let usage_percent = used as f64 * 100.0 / total as f64;

        assert!((usage_percent - 51.17).abs() < 0.01); // (16384000-8000000)/16384000
        assert_eq!(swap_total - swap_free, 194304 * 1024);
    }

    #[test]
    fn missing_available_falls_back() {
        let text = "MemTotal: 4096000 kB\nSwapTotal: 0 kB\n";
        let (total, avail, _, _) = parse_meminfo(text).unwrap();
        assert_eq!(total, 4096000 * 1024);
        assert_eq!(avail, total); // 无 MemAvailable -> 保守兜底
    }

    #[test]
    fn garbage_returns_none() {
        assert!(parse_meminfo("no numbers here").is_none());
    }
}
