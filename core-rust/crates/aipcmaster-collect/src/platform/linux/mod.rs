//! Linux 平台采集器。
//!
//! 数据来源（均为只读）：
//! - `/proc/stat`、`/proc/loadavg`：CPU；
//! - `/sys/class/thermal/`：温度；
//! - `/proc/meminfo`：内存；
//! - `/proc/mounts` + `statvfs(3)`：磁盘容量；
//! - `/proc/diskstats`：磁盘 I/O；
//! - `/proc/net/dev`：网络；
//! - `/proc/<pid>/`：进程。

mod cpu;
mod disk;
mod memory;
mod network;
mod process;

use std::time::{SystemTime, UNIX_EPOCH};

use crate::collector::SystemCollector;
use crate::error::Result;
use crate::metrics::SystemSnapshot;

/// Linux 采集器。
pub struct LinuxCollector {
    /// 有状态的 CPU 采样器（内部用 Mutex 维护基线，兼容 trait 的 `&self` 语义）。
    cpu: cpu::CpuSampler,
}

impl LinuxCollector {
    pub fn new() -> Self {
        Self {
            cpu: cpu::CpuSampler::default(),
        }
    }

    /// 核心采集逻辑。`full=true` 时额外采集进程列表与磁盘 I/O。
    fn collect_with(&self, full: bool) -> Result<SystemSnapshot> {
        let mut snap = SystemSnapshot {
            timestamp_unix_ms: unix_ms(),
            cpu: self.cpu.sample().ok(),
            memory: memory::sample(),
            disks: disk::capacity()?,
            networks: network::sample(),
            ..SystemSnapshot::default()
        };

        if full {
            snap.processes = process::list();
            snap.disk_io = disk::io_stats();
        }
        Ok(snap)
    }
}

impl Default for LinuxCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemCollector for LinuxCollector {
    fn collect(&self) -> Result<SystemSnapshot> {
        self.collect_with(false)
    }

    fn collect_full(&self) -> Result<SystemSnapshot> {
        self.collect_with(true)
    }
}

pub(crate) fn unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}
