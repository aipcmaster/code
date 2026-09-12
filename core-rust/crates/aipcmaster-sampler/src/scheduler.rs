//! 采样调度器实现。

use aipcmaster_collect::SystemCollector;
use aipcmaster_collect::SystemSnapshot;
use aipcmaster_diagnose::DiagnosticEngine;
use aipcmaster_diagnose::Issue;
use aipcmaster_diagnose::Severity;
use aipcmaster_store::AuditLog;
use aipcmaster_store::AuditLogKind;
use aipcmaster_store::Store;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::watch;

/// 采样调度错误。
#[derive(Debug, thiserror::Error)]
pub enum SamplerError {
    #[error("收集失败: {0}")]
    Collect(#[from] aipcmaster_collect::Error),
    #[error("存储失败: {0}")]
    Store(#[from] aipcmaster_store::Error),
}

/// 采样配置（对齐 PRD §5 硬性要求）。
#[derive(Debug, Clone, Copy)]
pub struct SamplerConfig {
    /// 正常采样间隔：5 秒。
    pub normal_interval: Duration,
    /// 异常采样间隔：500 毫秒。
    pub abnormal_interval: Duration,
    /// 从异常回退到正常所需的连续健康诊断次数（防抖）。
    pub healthy_debounce: u32,
}

impl Default for SamplerConfig {
    fn default() -> Self {
        Self {
            normal_interval: Duration::from_secs(5),
            abnormal_interval: Duration::from_millis(500),
            healthy_debounce: 3,
        }
    }
}

/// 一次采样的诊断摘要。
#[derive(Debug, Clone)]
pub struct SamplerReport {
    pub timestamp_unix_ms: u64,
    pub health_score: u8,
    pub issues: Vec<Issue>,
    pub sampling_mode: SamplingMode,
}

/// 当前采样档位。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SamplingMode {
    /// 正常档：5 秒一次。
    Normal,
    /// 异常档：500 毫秒一次。
    Abnormal,
}

/// 采样循环对外句柄：可停止、可查询状态、可订阅事件。
#[derive(Clone)]
pub struct SamplerHandle {
    stop: Arc<tokio::sync::Notify>,
    running: Arc<AtomicBool>,
    last_report: Arc<std::sync::Mutex<Option<SamplerReport>>>,
    events: watch::Receiver<Option<SamplerReport>>,
    samples_collected: Arc<AtomicU64>,
}

impl SamplerHandle {
    /// 请求停止采样循环（幂等，等待循环真正退出）。
    pub async fn stop(&self) {
        self.stop.notify_one();
        while self.running.load(Ordering::Acquire) {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }

    /// 是否仍在运行。
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Acquire)
    }

    /// 最近一次诊断报告。
    pub fn last_report(&self) -> Option<SamplerReport> {
        self.last_report.lock().ok().and_then(|g| g.clone())
    }

    /// 订阅后续诊断报告事件。
    pub fn subscribe(&self) -> watch::Receiver<Option<SamplerReport>> {
        self.events.clone()
    }

    /// 已采集快照总数。
    pub fn samples_collected(&self) -> u64 {
        self.samples_collected.load(Ordering::Relaxed)
    }
}

/// 采样调度器：采集 → 存储 → 诊断 → 频率自适应。
///
/// 频率切换规则：
/// - 出现 High/Critical 问题 → 切到异常档（500ms）；
/// - 连续 `healthy_debounce` 次无高风险 → 回正常档（5s）。
pub struct Sampler {
    collector: Box<dyn SystemCollector>,
    store: Store,
    engine: DiagnosticEngine,
    config: SamplerConfig,
}

impl Sampler {
    /// 构造采样器。
    pub fn new(collector: Box<dyn SystemCollector>, store: Store, config: SamplerConfig) -> Self {
        Self {
            collector,
            store,
            engine: DiagnosticEngine::new(),
            config,
        }
    }

    /// 开始采样循环。返回句柄用于停止/查询。
    pub fn run(self) -> SamplerHandle {
        let stop = Arc::new(tokio::sync::Notify::new());
        let running = Arc::new(AtomicBool::new(true));
        let last_report = Arc::new(std::sync::Mutex::new(None::<SamplerReport>));
        let samples_collected = Arc::new(AtomicU64::new(0));
        let (event_tx, event_rx) = watch::channel(None::<SamplerReport>);

        let handle = SamplerHandle {
            stop: stop.clone(),
            running: running.clone(),
            last_report: last_report.clone(),
            events: event_rx,
            samples_collected: samples_collected.clone(),
        };

        tokio::spawn(async move {
            let conn_store = self.store.clone();
            let mut current_mode = SamplingMode::Normal;
            let mut healthy_streak: u32 = 0;

            loop {
                if !running.load(Ordering::Acquire) {
                    break;
                }

                // 1. 采集
                let snap: SystemSnapshot = match self.collector.collect() {
                    Ok(s) => s,
                    Err(e) => {
                        let _ = conn_store.insert_audit(&AuditLog {
                            kind: AuditLogKind::Diagnose,
                            action: "collect_failed".into(),
                            target: None,
                            detail: e.to_string(),
                            created_at_unix_ms: now_ms(),
                        });
                        tokio::select! {
                            _ = sleep_dur(sleep_for(current_mode, &self.config)) => {}
                            _ = stop.notified() => {
                                running.store(false, Ordering::Release);
                                break;
                            }
                        }
                        continue;
                    }
                };
                samples_collected.fetch_add(1, Ordering::Relaxed);

                // 2. 存储（本地原始数据，断网可继续）
                let _ = conn_store.insert_snapshot(&snap);

                // 3. 诊断
                let report = self.engine.diagnose(&snap, &format!("sess-{}", now_ms()));

                // 4. 广播 + 频率自适应
                let out = SamplerReport {
                    timestamp_unix_ms: snap.timestamp_unix_ms,
                    health_score: report.health_score,
                    issues: report.issues.clone(),
                    sampling_mode: current_mode,
                };
                *last_report.lock().unwrap() = Some(out.clone());
                let _ = event_tx.send(Some(out));

                let has_high = report
                    .issues
                    .iter()
                    .any(|i| matches!(i.severity, Severity::High | Severity::Critical));

                if has_high {
                    healthy_streak = 0;
                    current_mode = SamplingMode::Abnormal;
                } else {
                    healthy_streak += 1;
                    if current_mode == SamplingMode::Abnormal
                        && healthy_streak >= self.config.healthy_debounce
                    {
                        current_mode = SamplingMode::Normal;
                        healthy_streak = 0;
                    }
                }

                // 5. 档位睡眠（可被 stop 打断）
                tokio::select! {
                    _ = sleep_dur(sleep_for(current_mode, &self.config)) => {}
                    _ = stop.notified() => {
                        running.store(false, Ordering::Release);
                        break;
                    }
                }
            }
        });

        handle
    }

    /// 同步采集一次（不启动循环），用于一次性诊断。
    pub fn collect_once(&self) -> Result<SystemSnapshot, SamplerError> {
        Ok(self.collector.collect()?)
    }
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// 按档位返回间隔（零值防御 → 回落默认）。
fn sleep_for(mode: SamplingMode, config: &SamplerConfig) -> Duration {
    match mode {
        SamplingMode::Normal => {
            if config.normal_interval.is_zero() {
                Duration::from_secs(5)
            } else {
                config.normal_interval
            }
        }
        SamplingMode::Abnormal => {
            if config.abnormal_interval.is_zero() {
                Duration::from_millis(500)
            } else {
                config.abnormal_interval
            }
        }
    }
}

async fn sleep_dur(d: Duration) {
    tokio::time::sleep(d).await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_matches_prd() {
        let cfg = SamplerConfig::default();
        assert_eq!(cfg.normal_interval, Duration::from_secs(5));
        assert_eq!(cfg.abnormal_interval, Duration::from_millis(500));
        assert_eq!(cfg.healthy_debounce, 3);
    }

    #[test]
    fn interval_fallbacks() {
        let cfg = SamplerConfig {
            normal_interval: Duration::ZERO,
            abnormal_interval: Duration::ZERO,
            healthy_debounce: 0,
        };
        assert_eq!(
            sleep_for(SamplingMode::Normal, &cfg),
            Duration::from_secs(5)
        );
        assert_eq!(
            sleep_for(SamplingMode::Abnormal, &cfg),
            Duration::from_millis(500)
        );
    }
}
