//! 磁盘采集：容量 `/proc/mounts` + `statvfs(3)`；I/O `/proc/diskstats`。

use crate::error::Result;
use crate::metrics::{DiskIoMetrics, DiskMetrics};

/// 关注的文件系统类型。
const WATCHED_FS: &[&str] = &[
    "ext2", "ext3", "ext4", "xfs", "btrfs", "ntfs", "ntfs3", "vfat", "exfat", "f2fs", "zfs",
];

/// 解析 `/proc/mounts` 文本中的挂载记录。
/// 行格式: device mountpoint fstype options dump pass
pub(crate) fn parse_mounts(text: &str) -> Vec<(String, String, String)> {
    let mut out = Vec::new();
    for line in text.lines() {
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() < 3 {
            continue;
        }
        // 只关心真实块设备挂载（跳过 tmpfs/proc/sysfs 等）
        if !fields[0].starts_with("/dev/") || !WATCHED_FS.contains(&fields[2]) {
            continue;
        }
        out.push((
            fields[0].to_string(),
            fields[1].to_string(),
            fields[2].to_string(),
        ));
    }
    out
}

/// 对挂载点调用 statvfs，返回 (total, used, avail) 字节。失败返回 None。
fn statvfs(mount_point: &str) -> Option<(u64, u64, u64)> {
    // 用 std::ffi::CString 构造 C 字符串
    let cpath = std::ffi::CString::new(mount_point).ok()?;
    let mut st: libc::statvfs = unsafe { std::mem::zeroed() };
    // SAFETY: cp 指向合法 C 字符串，st 指向合法缓冲区
    let ret = unsafe { libc::statvfs(cpath.as_ptr(), &mut st) };
    if ret != 0 {
        return None;
    }
    let frsize = st.f_frsize as u64;
    let total = (st.f_blocks as u64).saturating_mul(frsize);
    let free = (st.f_bfree as u64).saturating_mul(frsize);
    let used = total.saturating_sub(free);
    let avail = (st.f_bavail as u64).saturating_mul(frsize);
    Some((total, used, avail))
}

/// 读取挂载点容量指标（按设备去重，优先取根挂载）。
pub(crate) fn capacity() -> Result<Vec<DiskMetrics>> {
    let mounts_text =
        std::fs::read_to_string("/proc/mounts").map_err(|e| crate::error::Error::Io {
            context: "/proc/mounts",
            source: e,
        })?;
    let mounts = parse_mounts(&mounts_text);

    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::new();
    for (device, mount_point, fs_type) in mounts {
        if !seen.insert(device.clone()) {
            continue;
        }
        let Some((total, used, avail)) = statvfs(&mount_point) else {
            continue;
        };
        let usage_percent = if total == 0 {
            0.0
        } else {
            used as f64 * 100.0 / total as f64
        };
        out.push(DiskMetrics {
            device,
            mount_point,
            fs_type,
            total_bytes: total,
            used_bytes: used,
            available_bytes: avail,
            usage_percent,
        });
    }
    Ok(out)
}

/// 解析 `/proc/diskstats` 文本。
/// 行格式（字段 3 之后）: name reads reads_merged sectors_read time_read writes writes_merged sectors_written time_write ...
pub(crate) fn parse_diskstats(text: &str) -> Vec<DiskIoMetrics> {
    let mut out = Vec::new();
    for line in text.lines() {
        let f: Vec<&str> = line.split_whitespace().collect();
        if f.len() < 11 {
            continue;
        }
        let name = f[2];
        // 跳过虚拟设备（loop/ram/zram）
        if name.starts_with("loop") || name.starts_with("ram") || name.starts_with("zram") {
            continue;
        }
        let read_ops: u64 = f[3].parse().unwrap_or(0);
        let sectors_read: u64 = f[5].parse().unwrap_or(0);
        let write_ops: u64 = f[7].parse().unwrap_or(0);
        let sectors_written: u64 = f[9].parse().unwrap_or(0);
        out.push(DiskIoMetrics {
            device: name.to_string(),
            read_bytes: sectors_read.saturating_mul(512),
            write_bytes: sectors_written.saturating_mul(512),
            read_ops,
            write_ops,
        });
    }
    out
}

/// 读取磁盘 I/O 累计计数（仅完整快照时采集，属较轻量操作）。
pub(crate) fn io_stats() -> Vec<DiskIoMetrics> {
    std::fs::read_to_string("/proc/diskstats")
        .map(|t| parse_diskstats(&t))
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    const MOUNTS: &str = "\
/dev/sda2 / ext4 rw,relatime 0 0
/dev/sda1 /boot/efi vfat rw,relatime 0 0
proc /proc proc rw,nosuid 0 0
tmpfs /tmp tmpfs rw,nosuid 0 0
/dev/mapper/data /data btrfs rw,relatime 0 0
/dev/sda2 /var/bind ext4 rw,bind 0 0
";

    #[test]
    fn parses_mounts() {
        let mounts = parse_mounts(MOUNTS);
        assert_eq!(mounts.len(), 4); // sda2(ext4)、sda1(vfat)、mapper/btrfs、sda2绑定(ext4)
        assert!(mounts
            .iter()
            .any(|(d, m, t)| d == "/dev/sda2" && m == "/" && t == "ext4"));
        assert!(mounts
            .iter()
            .any(|(d, _, t)| d == "/dev/mapper/data" && t == "btrfs"));
        // 绑定的 sda2 /var/bind 应该被解析出来（statvfs 去重发生在 capacity 层）
        assert!(mounts
            .iter()
            .any(|(d, m, _)| d == "/dev/sda2" && m == "/var/bind"));
    }

    #[test]
    fn deviceless_mounts_skipped() {
        let text = "proc /proc proc rw 0 0\ntmpfs /tmp tmpfs rw 0 0\n";
        assert!(parse_mounts(text).is_empty());
    }

    const DISKSTATS: &str = "\
   8       0 sda 1000 50 20000 300 500 10 10000 150 0 30 40 50
   8       1 sda1 100 0 2000 30 50 0 1000 15 0 3 4 5
   7       0 loop0 0 0 0 0 0 0 0 0 0 0 0
 252       0 zram0 0 0 0 0 0 0 0 0 0 0 0
";

    #[test]
    fn parses_diskstats_skips_virtual() {
        let rows = parse_diskstats(DISKSTATS);
        assert_eq!(rows.len(), 2);

        let sda = &rows[0];
        assert_eq!(sda.device, "sda");
        assert_eq!(sda.read_ops, 1000);
        assert_eq!(sda.read_bytes, 20000 * 512);
        assert_eq!(sda.write_ops, 500);
        assert_eq!(sda.write_bytes, 10000 * 512);
    }
}
