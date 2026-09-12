//! 系统指标数据模型。
//!
//! 单位约定：
//! - 内存/磁盘/网络字节一律使用 SI 字节（bytes）；
//! - 百分比 0.0 ~ 100.0；
//! - 温度摄氏；
//! - 时间为 Unix 毫秒时间戳。

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// 正常采样间隔：5 秒（PRD §5 非功能需求）。
pub const NORMAL_SAMPLE_INTERVAL: Duration = Duration::from_secs(5);
/// 异常采样间隔：500 毫秒（PRD §5 非功能需求）。
pub const ABNORMAL_SAMPLE_INTERVAL: Duration = Duration::from_millis(500);

/// CPU 指标。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CpuMetrics {
    /// 整体使用率百分比（0.0 ~ 100.0）。首次采样无基线时为 0.0。
    pub usage_percent: f64,
    /// 每个逻辑核的使用率百分比。
    pub per_core_usage_percent: Vec<f64>,
    /// 负载均值（1/5/15 分钟）。
    pub load_avg_1: f64,
    pub load_avg_5: f64,
    pub load_avg_15: f64,
    /// 逻辑核数量。
    pub core_count: usize,
    /// CPU 封装温度（摄氏），不可用时为 None。
    pub temperature_c: Option<f64>,
}

/// 内存指标。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryMetrics {
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub available_bytes: u64,
    /// 使用率百分比（0.0 ~ 100.0）。
    pub usage_percent: f64,
    pub swap_total_bytes: u64,
    pub swap_used_bytes: u64,
}

/// 磁盘容量指标（某个挂载点）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiskMetrics {
    pub device: String,
    pub mount_point: String,
    pub fs_type: String,
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub available_bytes: u64,
    pub usage_percent: f64,
}

/// 磁盘 I/O 指标（自系统启动以来的累计计数，扇区按 512 字节计）。
/// 速率由上层对两次快照求差计算。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiskIoMetrics {
    pub device: String,
    pub read_bytes: u64,
    pub write_bytes: u64,
    pub read_ops: u64,
    pub write_ops: u64,
}

/// 网络接口指标（自系统启动以来的累计计数）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NetworkMetrics {
    pub interface: String,
    pub rx_bytes: u64,
    pub tx_bytes: u64,
    pub rx_errors: u64,
    pub tx_errors: u64,
}

/// 进程指标。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProcessMetrics {
    pub pid: i32,
    pub name: String,
    /// 进程状态字符（Linux: R/S/D/Z/T 等）。
    pub state: String,
    pub memory_bytes: u64,
}

/// 一次采集的系统快照。
///
/// - [`SystemCollector::collect`] 为轻量快照（5s 采样），不含进程列表；
/// - [`SystemCollector::collect_full`] 为完整快照（诊断/异常采样），含进程与磁盘 I/O。
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct SystemSnapshot {
    pub timestamp_unix_ms: u64,
    pub cpu: Option<CpuMetrics>,
    pub memory: Option<MemoryMetrics>,
    pub disks: Vec<DiskMetrics>,
    pub disk_io: Vec<DiskIoMetrics>,
    pub networks: Vec<NetworkMetrics>,
    pub processes: Vec<ProcessMetrics>,
}
