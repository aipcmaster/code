// 采集契约：与 core-rust SystemCollector trait 对齐。
// Collect() = 轻量快照（周期采样用）；诊断时通常需要完整快照。

namespace AIPCMaster.Core.Collectors;

public interface ISystemCollector
{
    /// 轻量快照（含 CPU/内存/磁盘，周期采样足够）。
    SystemSnapshot Collect();

    /// 完整快照（V1 Windows 实现与 Collect 同构；进程级采集待 PDH 接入）。
    SystemSnapshot CollectFull();
}