//! AIPCMaster（AI电脑大师）核心引擎 —— 采样调度层
//!
//! 对应《AIPCMaster（AI电脑大师）软件开发工程文档》§2.1「数据采集层」与 PRD §5：
//!
//! > 正常采样 5 秒，异常采样 500 毫秒。
//!
//! [`Sampler`] 是常驻后台任务：
//! 1. 以当前采样间隔采集一次 [`SystemSnapshot`]；
//! 2. 快照落入 [`Store`]（本地原始数据，断网可继续）；
//! 3. 交由 [`DiagnosticEngine`] 诊断；
//! 4. 按诊断结果切换采样档位：出现严重(high)及以上问题 → 异常档(500ms)，
//!    恢复健康（连续多次无高风险）→ 正常档(5s)。
//!
//! 设计取舍：采样循环**不阻塞**，一次采集失败只记录并继续（断点续采）；
//! 频率切换带防抖，避免抖动。

pub mod scheduler;

pub use scheduler::{
    Sampler, SamplerConfig, SamplerError, SamplerHandle, SamplerReport, SamplingMode,
};
