//! CPU 采集：`/proc/stat` + `/proc/loadavg` + `/sys/class/thermal`。

use std::sync::Mutex;

use crate::metrics::CpuMetrics;

/// `/proc/stat` 中某行（cpu 或 cpuN）的累计 jiffies。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CpuTimes {
    /// 总时间（user+nice+system+idle+iowait+irq+softirq+steal，不含 guest）。
    pub total: u64,
    /// 空闲时间（idle + iowait）。
    pub idle: u64,
}

/// 是否是 `/proc/stat` 中的核行名（cpu0、cpu1、...）。
fn is_cpu_core_name(name: &str) -> bool {
    name.len() > 3 && name.starts_with("cpu") && name[3..].chars().all(|c| c.is_ascii_digit())
}

/// 解析 `/proc/stat` 文本，提取总 CPU 与各核累计时间。
/// 每行字段: user nice system idle iowait irq softirq steal guest guest_nice
pub(crate) fn parse_stat(text: &str) -> Vec<(String, CpuTimes)> {
    let mut out = Vec::new();
    for line in text.lines() {
        let mut parts = line.split_whitespace();
        let Some(prefix) = parts.next() else { continue };
        if prefix != "cpu" && !is_cpu_core_name(prefix) {
            continue;
        }
        let fields: Vec<u64> = parts.map(|p| p.parse::<u64>().unwrap_or(0)).collect();
        // fields[0..8]: user nice system idle iowait irq softirq steal
        let (user, nice, system, idle, iowait, irq, softirq, steal) = match fields.as_slice() {
            [a, b, c, d, e, f, g, h, ..] => (*a, *b, *c, *d, *e, *f, *g, *h),
            _ => continue,
        };
        let total = user
            .wrapping_add(nice)
            .wrapping_add(system)
            .wrapping_add(idle)
            .wrapping_add(iowait)
            .wrapping_add(irq)
            .wrapping_add(softirq)
            .wrapping_add(steal);
        let idle = idle.wrapping_add(iowait);
        out.push((prefix.to_string(), CpuTimes { total, idle }));
    }
    out
}

/// 由两次累计值计算使用率（0.0~100.0）。无增量时返回 0.0。
pub(crate) fn usage_from_delta(prev: &CpuTimes, next: &CpuTimes) -> f64 {
    let total_delta = next.total.saturating_sub(prev.total);
    if total_delta == 0 {
        return 0.0;
    }
    let idle_delta = next.idle.saturating_sub(prev.idle);
    let busy_delta = total_delta.saturating_sub(idle_delta);
    (busy_delta as f64) * 100.0 / (total_delta as f64)
}

/// 单次基线： (总 CPU, 各核)
type CpuBaseline = (CpuTimes, Vec<(String, CpuTimes)>);

/// 有状态的 CPU 采样器。首次调用返回 0.0（无基线），后续返回真实使用率。
#[derive(Default)]
pub struct CpuSampler {
    inner: Mutex<Option<CpuBaseline>>,
}

impl CpuSampler {
    pub fn sample(&self) -> Result<CpuMetrics, &'static str> {
        let stat_text = std::fs::read_to_string("/proc/stat").map_err(|_| "read /proc/stat")?;
        let rows = parse_stat(&stat_text);
        if rows.is_empty() {
            return Err("no cpu rows in /proc/stat");
        }

        let total = rows
            .iter()
            .find(|(n, _)| n == "cpu")
            .map(|(_, t)| t.clone())
            .ok_or("missing aggregate cpu row")?;
        let cores: Vec<(String, CpuTimes)> = rows
            .into_iter()
            .filter(|(n, _)| is_cpu_core_name(n))
            .collect();
        let core_count = cores.len();

        let mut guard = self.inner.lock().map_err(|_| "cpu sampler lock poisoned")?;

        let (usage, per_core, prev_total, prev_cores) = match guard.take() {
            Some((prev_t, prev_c)) => {
                let usage = usage_from_delta(&prev_t, &total);
                let per_core: Vec<f64> = prev_c
                    .iter()
                    .map(|(name, pt)| {
                        cores
                            .iter()
                            .find(|(n, _)| n == name)
                            .map(|(_, nt)| usage_from_delta(pt, nt))
                            .unwrap_or(0.0)
                    })
                    .collect();
                (usage, per_core, total, cores)
            }
            None => (0.0, vec![0.0; core_count], total, cores),
        };

        *guard = Some((prev_total, prev_cores));

        let (load_avg_1, load_avg_5, load_avg_15) = loadavg().unwrap_or((0.0, 0.0, 0.0));

        Ok(CpuMetrics {
            usage_percent: usage,
            per_core_usage_percent: per_core,
            load_avg_1,
            load_avg_5,
            load_avg_15,
            core_count,
            temperature_c: temperature(),
        })
    }
}

/// 解析 `/proc/loadavg` 文本：`1min 5min 15min running/total pid`。
pub(crate) fn parse_loadavg(text: &str) -> Option<(f64, f64, f64)> {
    let mut it = text.split_whitespace();
    Some((
        it.next()?.parse().ok()?,
        it.next()?.parse().ok()?,
        it.next()?.parse().ok()?,
    ))
}

/// 读取 `/proc/loadavg`。
fn loadavg() -> Option<(f64, f64, f64)> {
    let text = std::fs::read_to_string("/proc/loadavg").ok()?;
    parse_loadavg(&text)
}

/// 解析某个 thermal_zone 的 temp 文件内容（毫摄氏度 → 摄氏）。
fn parse_temp_milli(milli: &str) -> Option<f64> {
    let m: f64 = milli.trim().parse().ok()?;
    Some(m / 1000.0)
}

/// 温度：扫描 `/sys/class/thermal/thermal_zone*/temp`，取最大值。
fn temperature() -> Option<f64> {
    let dir = std::fs::read_dir("/sys/class/thermal").ok()?;
    let mut max: Option<f64> = None;
    for entry in dir.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if !name.starts_with("thermal_zone") {
            continue;
        }
        let temp_path = entry.path().join("temp");
        let Ok(text) = std::fs::read_to_string(temp_path) else {
            continue;
        };
        let Some(c) = parse_temp_milli(&text) else {
            continue;
        };
        if max.is_none_or(|m| c > m) {
            max = Some(c);
        }
    }
    max
}

#[cfg(test)]
mod tests {
    use super::*;

    // 两次采样 fixture：
    // 整体（cpu）：
    //   S1: user1000 nice0 system500 idle8000 iowait0 irq100 softirq50 steal0
    //       total=9650, idle=8000, busy=1650
    //   S2: user1050 nice0 system525 idle8635 iowait0 irq100 softirq50 steal0
    //       total=10360, idle=8635, busy=1725
    //   delta: total=710, busy=75  -> usage = 75/710 ≈ 10.56%
    // cpu0:
    //   S1: 400 0 200 4000 0 50 25 0 -> total=4675, busy=675
    //   S2: 420 0 210 4087 0 50 25 0 -> total=4792, busy=705
    //   delta: total=117, busy=30  -> usage = 30/117 ≈ 25.64%
    // cpu1:
    //   S1: 600 0 300 4000 0 50 25 0 -> total=4975, busy=975
    //   S2: 630 0 315 4548 0 50 25 0 -> total=5568, busy=1020
    //   delta: total=593, busy=45  -> usage = 45/593 ≈ 7.59%

    const STAT_1: &str = "\
cpu  1000 0 500 8000 0 100 50 0 0 0
cpu0 400 0 200 4000 0 50 25 0 0 0
cpu1 600 0 300 4000 0 50 25 0 0 0
intr 12345
ctxt 67890
";

    const STAT_2: &str = "\
cpu  1050 0 525 8635 0 100 50 0 0 0
cpu0 420 0 210 4087 0 50 25 0 0 0
cpu1 630 0 315 4548 0 50 25 0 0 0
intr 12346
ctxt 67891
";

    #[test]
    fn parses_cpu_rows() {
        let rows = parse_stat(STAT_1);
        assert_eq!(rows.len(), 3); // cpu + cpu0 + cpu1

        let (name, t) = &rows[0];
        assert_eq!(name, "cpu");
        assert_eq!(t.total, 9650);
        assert_eq!(t.idle, 8000);
    }

    #[test]
    fn parse_stat_ignores_non_cpu_lines() {
        let rows = parse_stat("cpu 1 2 3 4 5 6 7 8\nintr 999\ndummy 1 2\ncpu0 1 1 1 1 1 1 1 1\n");
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].0, "cpu");
        assert_eq!(rows[1].0, "cpu0");
    }

    #[test]
    fn usage_from_deltas() {
        let s1 = parse_stat(STAT_1);
        let s2 = parse_stat(STAT_2);

        let u = usage_from_delta(&s1[0].1, &s2[0].1);
        assert!((u - 10.56).abs() < 0.05, "overall usage {u}");

        let u0 = usage_from_delta(&s1[1].1, &s2[1].1);
        assert!((u0 - 25.64).abs() < 0.05, "cpu0 usage {u0}");

        let u1 = usage_from_delta(&s1[2].1, &s2[2].1);
        assert!((u1 - 7.59).abs() < 0.05, "cpu1 usage {u1}");
    }

    #[test]
    fn usage_zero_on_no_delta() {
        let t = CpuTimes {
            total: 1000,
            idle: 800,
        };
        assert_eq!(usage_from_delta(&t, &t), 0.0);
    }

    #[test]
    fn loadavg_parsing() {
        let v = parse_loadavg("0.52 0.58 0.59 2/458 12345\n");
        assert!(v.is_some());
        let (a, b, c) = v.unwrap();
        assert!((a - 0.52).abs() < 1e-9);
        assert!((b - 0.58).abs() < 1e-9);
        assert!((c - 0.59).abs() < 1e-9);

        assert!(parse_loadavg("garbage").is_none());
        assert!(parse_loadavg("").is_none());
    }

    #[test]
    fn temp_parsing() {
        let c = parse_temp_milli("58600\n");
        assert!((c.unwrap() - 58.6).abs() < 1e-9);
        assert!(parse_temp_milli("abc").is_none());
    }
}
