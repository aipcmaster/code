//! 网络采集：`/proc/net/dev`。

use crate::metrics::NetworkMetrics;

/// 解析 `/proc/net/dev` 文本。
/// 首行是表头（"Inter-| Receive ..."），第二行是列名行，之后每行一个接口。
/// 行格式: iface: rx_bytes rx_packets rx_errs rx_drop ... tx_bytes tx_packets tx_errs tx_drop ...
pub(crate) fn parse_net_dev(text: &str) -> Vec<NetworkMetrics> {
    let mut out = Vec::new();
    for line in text.lines().skip(2) {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let Some((name, rest)) = line.split_once(':') else {
            continue;
        };
        let fields: Vec<u64> = rest
            .split_whitespace()
            .map(|p| p.parse().unwrap_or(0))
            .collect();
        if fields.len() < 9 {
            continue;
        }
        // rx: bytes, packets, errs, drop, fifo, frame, compressed, multicast (8)
        // tx: bytes, packets, errs, drop, fifo, colls, carrier, compressed (8)
        let rx_bytes = fields[0];
        let rx_errors = fields[2];
        let tx_bytes = fields[8];
        let tx_errors = fields[10];
        out.push(NetworkMetrics {
            interface: name.to_string(),
            rx_bytes,
            tx_bytes,
            rx_errors,
            tx_errors,
        });
    }
    out
}

/// 读取当前网络接口累计计数。
pub(crate) fn sample() -> Vec<NetworkMetrics> {
    std::fs::read_to_string("/proc/net/dev")
        .map(|t| parse_net_dev(&t))
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    const NET_DEV: &str = "\
Inter-|   Receive                                                |  Transmit
 face |bytes    packets errs drop fifo frame compressed multicast|bytes    packets errs drop fifo colls carrier compressed
    lo: 1000000   10000    0    0    0     0          0         0  1000000   10000    0    0    0     0       0          0
 enp3s0: 500000000  500000    3    0    0     0          0         0  200000000  200000    1    0    0     0       0          0
";

    #[test]
    fn parses_interfaces() {
        let rows = parse_net_dev(NET_DEV);
        assert_eq!(rows.len(), 2);

        let lo = &rows[0];
        assert_eq!(lo.interface, "lo");
        assert_eq!(lo.rx_bytes, 1000000);
        assert_eq!(lo.tx_bytes, 1000000);
        assert_eq!(lo.rx_errors, 0);

        let eth = &rows[1];
        assert_eq!(eth.interface, "enp3s0");
        assert_eq!(eth.rx_errors, 3);
        assert_eq!(eth.tx_errors, 1);
    }

    #[test]
    fn malformed_lines_skipped() {
        let text = "Inter-| Receive | Transmit\n face |bytes |tt\nnot an interface line\n  : 1 2\n";
        // 前两行被跳过，剩下两行均无法解析
        let rows = parse_net_dev(text);
        assert!(rows.is_empty());
    }
}
