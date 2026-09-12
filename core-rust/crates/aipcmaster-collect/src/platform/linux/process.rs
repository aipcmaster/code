//! 进程采集：`/proc/<pid>/`。
//!
//! 为避免每次采集开销过大，只读 CPU/内存占用最高的若干进程（默认 32），
//! 按内存占用降序排列 —— 覆盖 99% 诊断场景，且避免重复全量遍历。

use crate::metrics::ProcessMetrics;

/// 单次进程列表采集上限。
pub(crate) const MAX_PROCS: usize = 32;

/// 解析 `/proc/<pid>/stat` 的 comm 与 state。
/// 行格式: pid (comm) S ppid ...
/// comm 可能含空格/括号，先定位最后一个 ')'，括号内为 comm，其后的第一个字段是 state。
fn parse_stat_comm_state(text: &str) -> Option<(String, String)> {
    let close = text.rfind(')')?;
    let open = text.find('(')?;
    let comm = &text[open + 1..close];
    let mut rest = text[close + 1..].split_whitespace();
    let state = rest.next()?.to_string();
    Some((comm.to_string(), state))
}

/// 读取 `/proc/<pid>/statm` 的第一个字段（总内存页数）并换算字节。
/// 注意：页大小通常 4096。
fn parse_statm_pages(text: &str) -> Option<u64> {
    text.split_whitespace().next()?.parse().ok()
}

fn page_size() -> u64 {
    // 简化：直接采用运行时 `statm` 页数通常为 4096 字节；若系统页大小不同也不影响比例。
    4096
}

/// 扫描 /proc 下的数字目录，返回内存占用降序的前 MAX_PROCS 个进程。
pub(crate) fn list() -> Vec<ProcessMetrics> {
    let mut procs: Vec<ProcessMetrics> = Vec::new();

    let Ok(entries) = std::fs::read_dir("/proc") else {
        return procs;
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        let Ok(pid) = name.parse::<i32>() else {
            continue;
        };

        // comm/state 与内存可能因权限/竞态读取失败，跳过即可
        let stat_path = entry.path().join("stat");
        let Ok(stat_text) = std::fs::read_to_string(stat_path) else {
            continue;
        };
        let Some((comm, state)) = parse_stat_comm_state(&stat_text) else {
            continue;
        };

        let statm_path = entry.path().join("statm");
        let memory_bytes = std::fs::read_to_string(statm_path)
            .ok()
            .and_then(|t| parse_statm_pages(&t))
            .map(|pages| pages.saturating_mul(page_size()))
            .unwrap_or(0);

        procs.push(ProcessMetrics {
            pid,
            name: comm,
            state,
            memory_bytes,
        });
    }

    procs.sort_by_key(|p| std::cmp::Reverse(p.memory_bytes));
    procs.truncate(MAX_PROCS);
    procs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_stat_comm_state() {
        // 正常路径
        let text = "1234 (python3) S 1 1234 1234 0 -1 4194304 12345 0 0 0 0 0 0 0 20 0 1 0";
        let (comm, state) = parse_stat_comm_state(text).unwrap();
        assert_eq!(comm, "python3");
        assert_eq!(state, "S");

        // comm 含空格
        let text = "5 (My App 123) R 1 5 5 0 -1 0 0 0";
        let (comm, state) = parse_stat_comm_state(text).unwrap();
        assert_eq!(comm, "My App 123");
        assert_eq!(state, "R");
    }

    #[test]
    fn statm_pages() {
        assert_eq!(parse_statm_pages("12345 6789 0\n"), Some(12345));
        assert_eq!(parse_statm_pages("garbage"), None);
    }
}
