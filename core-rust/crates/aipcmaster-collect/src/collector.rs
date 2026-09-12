//! 采集接口定义。

use crate::error::Result;
use crate::metrics::SystemSnapshot;

/// 系统采集器 trait。
///
/// 各平台后端实现本 trait：
/// - Linux：读取 /proc、/sys（见 [`crate::platform::linux`]）；
/// - Windows / macOS：后续版本实现（原生 API）。
pub trait SystemCollector: Send + Sync {
    /// 轻量快照：CPU、内存、磁盘容量、网络、温度。
    /// 对应 5 秒正常采样。
    fn collect(&self) -> Result<SystemSnapshot>;

    /// 完整快照：轻量快照 + 进程列表 + 磁盘 I/O。
    /// 对应 500 毫秒异常采样与诊断触发。
    fn collect_full(&self) -> Result<SystemSnapshot>;
}
