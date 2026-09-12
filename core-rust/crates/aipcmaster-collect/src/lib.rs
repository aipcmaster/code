//! AIPCMaster（AI电脑大师）核心引擎 —— 数据采集层
//!
//! 对应《AIPCMaster（AI电脑大师）软件开发工程文档》§2.1：
//! - 采集 CPU、内存、磁盘、网络、温度、进程；
//! - 正常采样 5 秒，异常采样 500 毫秒；
//! - 本地缓存，断网可继续工作（采集层本身不依赖网络）。
//!
//! 设计原则：跨平台 trait + 平台后端（当前实现 Linux /proc、/sys）。
//! 本层只读，不做任何系统修改（决策执行层负责优化与回滚）。

pub mod collector;
pub mod error;
pub mod metrics;
mod platform;

pub use collector::SystemCollector;
pub use error::{Error, Result};
pub use metrics::{
    CpuMetrics, DiskIoMetrics, DiskMetrics, MemoryMetrics, NetworkMetrics, ProcessMetrics,
    SystemSnapshot, ABNORMAL_SAMPLE_INTERVAL, NORMAL_SAMPLE_INTERVAL,
};

/// 构建当前平台默认的采集器。
///
/// 未支持平台返回 [`Error::UnsupportedPlatform`]。
pub fn system_collector() -> Result<Box<dyn SystemCollector>> {
    platform::default()
}
