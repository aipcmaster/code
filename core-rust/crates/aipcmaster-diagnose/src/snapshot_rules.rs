//! 基于快照的标准诊断规则集。
//!
//! 阈值（V1 经验值，后续可配置化 / 由 AI 层校准）：
//! - CPU 使用率 > 90% → High
//! - 内存使用率 > 90% → High；> 75% → Medium
//! - 交换分区使用率 > 50% → Medium
//! - 磁盘使用率 > 95% → Critical；> 85% → High
//! - CPU 温度 > 95°C → Critical；> 80°C → High
//! - 1 分钟负载 > 核数 × 2 → Medium（整体繁忙）

use crate::issue::{Category, Issue, Severity};
use crate::rule::{DiagnosticRule, FnRule, RuleVerdict};
use aipcmaster_collect::SystemSnapshot;
use serde_json::json;

/// 返回全部标准规则。
pub fn standard_rules() -> Vec<Box<dyn DiagnosticRule>> {
    vec![
        Box::new(FnRule::new("cpu-usage", cpu_usage)),
        Box::new(FnRule::new("memory-usage", memory_usage)),
        Box::new(FnRule::new("swap-usage", swap_usage)),
        Box::new(FnRule::new("disk-usage", disk_usage)),
        Box::new(FnRule::new("cpu-temperature", cpu_temperature)),
        Box::new(FnRule::new("load-average", load_average)),
    ]
}

fn cpu_usage(s: &SystemSnapshot) -> RuleVerdict {
    let Some(cpu) = &s.cpu else { return None };
    if cpu.usage_percent >= 90.0 {
        Some(Issue::new(
            Category::Performance,
            Severity::High,
            "CPU 使用率持续偏高",
            "CPU 平均使用率超过 90%，可能由后台任务或异常进程导致，建议排查占用最高的进程。",
            json!({ "cpu_percent": cpu.usage_percent }),
        ))
    } else {
        None
    }
}

fn memory_usage(s: &SystemSnapshot) -> RuleVerdict {
    let Some(mem) = &s.memory else { return None };
    if mem.usage_percent >= 90.0 {
        Some(Issue::new(
            Category::Memory,
            Severity::High,
            "内存占用偏高",
            "内存使用率超过 90%，可能出现卡顿，建议清理后台进程或检查内存泄漏。",
            json!({ "mem_percent": mem.usage_percent }),
        ))
    } else if mem.usage_percent >= 75.0 {
        Some(Issue::new(
            Category::Memory,
            Severity::Medium,
            "内存占用偏高（轻度）",
            "内存使用率超过 75%，建议关注内存大户。",
            json!({ "mem_percent": mem.usage_percent }),
        ))
    } else {
        None
    }
}

fn swap_usage(s: &SystemSnapshot) -> RuleVerdict {
    let Some(mem) = &s.memory else { return None };
    if mem.swap_total_bytes == 0 {
        return None;
    }
    let used = mem.swap_used_bytes as f64 / mem.swap_total_bytes as f64 * 100.0;
    if used >= 50.0 {
        Some(Issue::new(
            Category::Memory,
            Severity::Medium,
            "交换分区（Swap）使用过半",
            "Swap 使用率超过 50%，物理内存可能不足，频繁换页会造成卡顿。",
            json!({ "swap_percent": used }),
        ))
    } else {
        None
    }
}

fn disk_usage(s: &SystemSnapshot) -> RuleVerdict {
    // 只检查容量盘（排除 tmpfs 等）；只报最满的一块盘
    let worst = s
        .disks
        .iter()
        .filter(|d| {
            !d.device.contains("loop") && !d.device.contains("zram") && !d.device.contains("ram")
        })
        .max_by(|a, b| a.usage_percent.total_cmp(&b.usage_percent))?;
    let p = worst.usage_percent;
    if p >= 95.0 {
        Some(Issue::new(
            Category::Disk,
            Severity::Critical,
            "磁盘空间即将占满",
            "磁盘使用率超过 95%，继续使用将导致系统异常，建议立即清理。",
            json!({ "device": worst.device, "percent": p }),
        ))
    } else if p >= 85.0 {
        Some(Issue::new(
            Category::Disk,
            Severity::High,
            "磁盘空间不足",
            "磁盘使用率超过 85%，建议清理缓存和大文件。",
            json!({ "device": worst.device, "percent": p }),
        ))
    } else {
        None
    }
}

fn cpu_temperature(s: &SystemSnapshot) -> RuleVerdict {
    let Some(cpu) = &s.cpu else { return None };
    let temp = cpu.temperature_c?;
    if temp >= 95.0 {
        Some(Issue::new(
            Category::Performance,
            Severity::Critical,
            "CPU 温度过高",
            "CPU 温度超过 95°C，可能降频或损坏硬件，请检查散热。",
            json!({ "temp_c": temp }),
        ))
    } else if temp >= 80.0 {
        Some(Issue::new(
            Category::Performance,
            Severity::High,
            "CPU 温度偏高",
            "CPU 温度超过 80°C，建议清理风扇或降低负载。",
            json!({ "temp_c": temp }),
        ))
    } else {
        None
    }
}

fn load_average(s: &SystemSnapshot) -> RuleVerdict {
    let Some(cpu) = &s.cpu else { return None };
    let cores = cpu.core_count.max(1) as f64;
    if cpu.load_avg_1 >= cores * 2.0 {
        Some(Issue::new(
            Category::Performance,
            Severity::Medium,
            "系统整体繁忙",
            "1 分钟平均负载达到核心数的 2 倍以上，系统可能接近过载。",
            json!({ "load_avg_1": cpu.load_avg_1, "cores": cpu.core_count }),
        ))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aipcmaster_collect::{
        CpuMetrics, DiskMetrics, MemoryMetrics, NetworkMetrics, ProcessMetrics, SystemSnapshot,
    };

    /// 全健康基线：CPU 5%、内存 25%、Swap 0%、磁盘 20%、温度 40°C、负载 0.5/8 核。
    fn base() -> SystemSnapshot {
        SystemSnapshot {
            timestamp_unix_ms: 0,
            cpu: Some(CpuMetrics {
                usage_percent: 5.0,
                per_core_usage_percent: vec![5.0; 8],
                load_avg_1: 0.5,
                load_avg_5: 0.4,
                load_avg_15: 0.3,
                core_count: 8,
                temperature_c: Some(40.0),
            }),
            memory: Some(MemoryMetrics {
                total_bytes: 16 << 30,
                used_bytes: 4 << 30,
                available_bytes: 12 << 30,
                usage_percent: 25.0,
                swap_total_bytes: 8 << 30,
                swap_used_bytes: 0,
            }),
            disks: vec![DiskMetrics {
                device: "/dev/sda1".into(),
                mount_point: "/".into(),
                fs_type: "ext4".into(),
                total_bytes: 500_000_000,
                used_bytes: 100_000_000,
                available_bytes: 400_000_000,
                usage_percent: 20.0,
            }],
            disk_io: vec![],
            networks: vec![NetworkMetrics {
                interface: "eth0".into(),
                rx_bytes: 0,
                tx_bytes: 0,
                rx_errors: 0,
                tx_errors: 0,
            }],
            processes: vec![ProcessMetrics {
                pid: 1,
                name: "init".into(),
                state: "S".into(),
                memory_bytes: 0,
            }],
        }
    }

    #[test]
    fn healthy_base_no_verdicts() {
        let rules = standard_rules();
        let s = base();
        for (i, r) in rules.iter().enumerate() {
            assert!(r.check(&s).is_none(), "第 {} 条规则误报", i + 1);
        }
    }

    fn set_cpu(s: &mut SystemSnapshot, usage: f64, temp: f64, load: f64) {
        let c = s.cpu.as_mut().unwrap();
        c.usage_percent = usage;
        c.temperature_c = Some(temp);
        c.load_avg_1 = load;
    }

    #[test]
    fn cpu_high_triggers_high() {
        let mut s = base();
        set_cpu(&mut s, 95.0, 40.0, 0.5);
        let v = cpu_usage(&s).unwrap();
        assert_eq!(v.severity, Severity::High);
        assert_eq!(v.category, Category::Performance);
    }

    #[test]
    fn cpu_ok_at_89() {
        let mut s = base();
        set_cpu(&mut s, 89.9, 40.0, 0.5);
        assert!(cpu_usage(&s).is_none());
    }

    #[test]
    fn memory_tiers() {
        let mut s = base();
        s.memory.as_mut().unwrap().usage_percent = 76.0;
        assert_eq!(memory_usage(&s).unwrap().severity, Severity::Medium);

        s.memory.as_mut().unwrap().usage_percent = 92.0;
        assert_eq!(memory_usage(&s).unwrap().severity, Severity::High);
    }

    #[test]
    fn swap_tier() {
        let mut s = base();
        s.memory.as_mut().unwrap().swap_used_bytes = 4u64 << 30; // 8GB 的 50%
        assert_eq!(swap_usage(&s).unwrap().severity, Severity::Medium);

        let mut no_swap = base();
        no_swap.memory.as_mut().unwrap().swap_total_bytes = 0;
        assert!(swap_usage(&no_swap).is_none());
    }

    #[test]
    fn disk_tiers() {
        let mut s = base();
        s.disks[0].usage_percent = 86.0;
        assert_eq!(disk_usage(&s).unwrap().severity, Severity::High);

        s.disks[0].usage_percent = 96.0;
        assert_eq!(disk_usage(&s).unwrap().severity, Severity::Critical);
    }

    #[test]
    fn temp_tiers() {
        let mut s = base();
        s.cpu.as_mut().unwrap().temperature_c = Some(82.0);
        assert_eq!(cpu_temperature(&s).unwrap().severity, Severity::High);

        s.cpu.as_mut().unwrap().temperature_c = Some(97.0);
        assert_eq!(cpu_temperature(&s).unwrap().severity, Severity::Critical);
    }

    #[test]
    fn load_tier() {
        let mut s = base();
        s.cpu.as_mut().unwrap().load_avg_1 = 17.0; // 8 核 × 2 = 16 阈值
        assert!(load_average(&s).is_some());
    }
}
