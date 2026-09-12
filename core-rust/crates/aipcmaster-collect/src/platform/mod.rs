//! 平台后端。
//!
//! 当前实现 Linux（/proc、/sys,零守护进程依赖）。Windows / macOS 后补。

use crate::collector::SystemCollector;
use crate::error::Result;

#[cfg(target_os = "linux")]
mod linux;

/// 构建当前平台默认采集器。
pub(crate) fn default() -> Result<Box<dyn SystemCollector>> {
    #[cfg(target_os = "linux")]
    {
        Ok(Box::new(linux::LinuxCollector::new()))
    }
    #[cfg(not(target_os = "linux"))]
    {
        Err(Error::UnsupportedPlatform(std::env::consts::OS))
    }
}
