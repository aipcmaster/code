//! 采样调度器集成测试：注入可控 MockCollector，验证
//! 采样循环、存储落库、频率自适应（5s ↔ 500ms）、句柄停止。

use aipcmaster_collect::{
    CpuMetrics, MemoryMetrics, SystemCollector, SystemSnapshot,
};
use aipcmaster_sampler::{Sampler, SamplerConfig, SamplingMode};
use aipcmaster_store::Store;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// 可控采集器：返回预先设定的快照；可记录调用次数。
#[derive(Clone)]
struct MockCollector {
    calls: Arc<AtomicU64>,
    healthy: bool,
}

impl MockCollector {
    fn new(healthy: bool) -> Self {
        Self {
            calls: Arc::new(AtomicU64::new(0)),
            healthy,
        }
    }
}

impl SystemCollector for MockCollector {
    fn collect(&self) -> aipcmaster_collect::Result<SystemSnapshot> {
        self.calls.fetch_add(1, Ordering::Relaxed);
        Ok(snapshot(self.healthy))
    }

    fn collect_full(&self) -> aipcmaster_collect::Result<SystemSnapshot> {
        self.collect()
    }
}

fn snapshot(healthy: bool) -> SystemSnapshot {
    let usage = if healthy { 25.0 } else { 94.0 };
    SystemSnapshot {
        timestamp_unix_ms: now_ms(),
        cpu: Some(CpuMetrics {
            usage_percent: if healthy { 5.0 } else { 95.0 },
            per_core_usage_percent: vec![],
            load_avg_1: if healthy { 0.5 } else { 30.0 },
            load_avg_5: 0.5,
            load_avg_15: 0.5,
            core_count: 8,
            temperature_c: None,
        }),
        memory: Some(MemoryMetrics {
            total_bytes: 16_000_000_000,
            used_bytes: if healthy { 4_000_000_000 } else { 15_000_000_000 },
            available_bytes: if healthy { 12_000_000_000 } else { 1_000_000_000 },
            usage_percent: usage,
            swap_total_bytes: 8_000_000_000,
            swap_used_bytes: 0,
        }),
        ..Default::default()
    }
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

#[tokio::test]
async fn sampler_runs_and_stops() {
    let store = Store::open_in_memory().unwrap();
    let collector = MockCollector::new(true); // 健康系统
    let cfg = SamplerConfig {
        normal_interval: Duration::from_millis(20), // 测试提速
        abnormal_interval: Duration::from_millis(5),
        ..Default::default()
    };

    let sampler = Sampler::new(Box::new(collector.clone()), store.clone(), cfg);
    let handle = sampler.run();

    // 跑一小段，等待至少两次采样
    tokio::time::sleep(Duration::from_millis(80)).await;

    assert!(handle.is_running());
    assert!(collector.calls.load(Ordering::Relaxed) >= 2);
    assert!(handle.samples_collected() >= 2);

    // 快照已落库
    assert!(store.latest_snapshot().unwrap().is_some());

    // 上次报告存在且为正常档
    let report = handle.last_report().expect("应有报告");
    assert_eq!(report.sampling_mode, SamplingMode::Normal);

    // 停止
    handle.stop().await;
    assert!(!handle.is_running());
}

#[tokio::test]
async fn abnormal_mode_when_sick() {
    let store = Store::open_in_memory().unwrap();
    let collector = MockCollector::new(false); // 高风险系统
    let cfg = SamplerConfig {
        normal_interval: Duration::from_millis(100),
        abnormal_interval: Duration::from_millis(5),
        healthy_debounce: 2,
    };

    let sampler = Sampler::new(Box::new(collector), store, cfg);
    let handle = sampler.run();

    tokio::time::sleep(Duration::from_millis(80)).await;

    let report = handle.last_report().expect("应有报告");
    assert_eq!(report.sampling_mode, SamplingMode::Abnormal);
    assert!(!report.issues.is_empty());

    handle.stop().await;
}