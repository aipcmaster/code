//! 真实系统冒烟测试：在 Linux 上对当前主机采集一次快照，验证整条链路。
//!
//! 不校验具体数值（不同机器差异大），只校验结构合理性：CPU/内存存在、磁盘非空、
//! 完整快照含进程列表且有内存占用排序。

use aipcmaster_collect::system_collector;

#[test]
#[cfg(target_os = "linux")]
fn smoke_lightweight_snapshot() {
    let collector = system_collector().expect("默认采集器构建失败（Linux）");
    let snap = collector.collect().expect("轻量快照采集失败");

    assert!(snap.timestamp_unix_ms > 0, "时间戳缺失");
    assert!(snap.cpu.is_some(), "CPU 指标缺失");
    let cpu = snap.cpu.as_ref().unwrap();
    assert!(cpu.core_count >= 1, "逻辑核数量异常");
    assert_eq!(cpu.per_core_usage_percent.len(), cpu.core_count);

    assert!(snap.memory.is_some(), "内存指标缺失");
    let mem = snap.memory.as_ref().unwrap();
    assert!(mem.total_bytes > 0);
    assert!((0.0..=100.0).contains(&mem.usage_percent));

    assert!(!snap.disks.is_empty(), "磁盘指标为空");
    for d in &snap.disks {
        assert!(d.total_bytes > 0);
        assert!((0.0..=100.0).contains(&d.usage_percent));
    }

    // 轻量快照不应包含进程列表
    assert!(snap.processes.is_empty());
}

#[test]
#[cfg(target_os = "linux")]
fn smoke_full_snapshot() {
    let collector = system_collector().expect("默认采集器构建失败（Linux）");
    let snap = collector.collect_full().expect("完整快照采集失败");

    assert!(!snap.processes.is_empty(), "进程列表为空");
    // 按内存降序
    let mut sorted = snap.processes.clone();
    sorted.sort_by_key(|p| std::cmp::Reverse(p.memory_bytes));
    assert_eq!(snap.processes, sorted);
}
