// Windows 采集后端：纯 P/Invoke（kernel32）+ System.IO.DriveInfo。
//
// 设计约束：不依赖任何 NuGet 包（离线环境），因此不用 PerformanceCounter/WMI，
// 而用 Win32 API——GetSystemTimes（CPU 双采样差分，与 Linux /proc/stat 同构）、
// GlobalMemoryStatusEx（内存）、GetSystemInfo（核数）、DriveInfo（磁盘容量）。
//
// CPU 温度在 Windows 上无公开 Win32 API（需 WMI/驱动），返回 null → 温度规则自动跳过。
// Windows 无 loadavg，LoadAvg* 返回 null → 负载规则自动跳过。

using System.Runtime.InteropServices;

namespace AIPCMaster.Core.Collectors;

public sealed class WindowsSystemCollector : ISystemCollector
{
    // ---- kernel32 P/Invoke ----

    [StructLayout(LayoutKind.Sequential)]
    private struct FILETIME
    {
        public uint DwLowDateTime;
        public uint DwHighDateTime;

        public ulong ToUInt64() => ((ulong)DwHighDateTime << 32) | DwLowDateTime;
    }

    [StructLayout(LayoutKind.Sequential)]
    private struct MEMORYSTATUSEX
    {
        public uint DwLength;
        public uint DwMemoryLoad;
        public ulong UllTotalPhys;
        public ulong UllAvailPhys;
        public ulong UllTotalPageFile;
        public ulong UllAvailPageFile;
        public ulong UllTotalVirtual;
        public ulong UllAvailVirtual;
        public ulong UllAvailExtendedVirtual;
    }

    [StructLayout(LayoutKind.Sequential)]
    private struct SYSTEM_INFO
    {
        public ushort ProcessorArchitecture;
        public ushort Reserved;
        public uint PageSize;
        public IntPtr MinimumApplicationAddress;
        public IntPtr MaximumApplicationAddress;
        public IntPtr ActiveProcessorMask;
        public uint NumberOfProcessors;
        public uint ProcessorType;
        public uint AllocationGranularity;
        public ushort ProcessorLevel;
        public ushort ProcessorRevision;
    }

    [DllImport("kernel32.dll", SetLastError = false)]
    private static extern bool GetSystemTimes(
        out FILETIME idleTime, out FILETIME kernelTime, out FILETIME userTime);

    [DllImport("kernel32.dll", SetLastError = false)]
    private static extern void GlobalMemoryStatusEx(ref MEMORYSTATUSEX buffer);

    [DllImport("kernel32.dll", SetLastError = false)]
    private static extern void GetSystemInfo(out SYSTEM_INFO lpSystemInfo);

    // ---- 状态 ----

    private readonly object _lock = new();
    private ulong _prevIdle;
    private ulong _prevKernel;
    private ulong _prevUser;

    public SystemSnapshot Collect() => CollectCore();

    public SystemSnapshot CollectFull() => CollectCore();

    private SystemSnapshot CollectCore()
    {
        var cpu = ReadCpu();
        var mem = ReadMemory();
        var disks = ReadDisks();

        return new SystemSnapshot
        {
            TimestampUnixMs = DateTimeOffset.UtcNow.ToUnixTimeMilliseconds(),
            Cpu = cpu,
            Memory = mem,
            Disks = disks,
        };
    }

    private CpuMetrics ReadCpu()
    {
        if (!GetSystemTimes(out var idle, out var kernel, out var user))
        {
            return new CpuMetrics { CoreCount = Environment.ProcessorCount };
        }

        ulong idleNow = idle.ToUInt64();
        ulong kernelNow = kernel.ToUInt64();
        ulong userNow = user.ToUInt64();

        lock (_lock)
        {
            double usage = 0.0;
            if (_prevKernel != 0 || _prevUser != 0)
            {
                ulong totalNow = kernelNow + userNow;
                ulong totalPrev = _prevKernel + _prevUser;
                ulong idlePrev = _prevIdle;

                ulong totalDelta = totalNow - totalPrev;
                ulong idleDelta = idleNow - idlePrev;
                if (totalDelta > 0)
                {
                    // kernel 已包含 idle；非空闲 = (kernel+user) - idle
                    usage = 100.0 * (1.0 - (double)idleDelta / totalDelta);
                }
            }

            _prevIdle = idleNow;
            _prevKernel = kernelNow;
            _prevUser = userNow;
            return new CpuMetrics
            {
                UsagePercent = Math.Clamp(usage, 0.0, 100.0),
                PerCoreUsagePercent = [],
                CoreCount = Environment.ProcessorCount,
                LoadAvg1 = null,
                LoadAvg5 = null,
                LoadAvg15 = null,
                TemperatureC = null,
            };
        }
    }

    private static MemoryMetrics ReadMemory()
    {
        var status = new MEMORYSTATUSEX { DwLength = (uint)Marshal.SizeOf<MEMORYSTATUSEX>() };
        GlobalMemoryStatusEx(ref status);

        ulong total = status.UllTotalPhys;
        ulong avail = status.UllAvailPhys;
        double usage = total == 0 ? 100.0 : 100.0 * (double)(total - avail) / total;

        return new MemoryMetrics
        {
            TotalBytes = total,
            UsedBytes = total - avail,
            AvailableBytes = avail,
            UsagePercent = Math.Clamp(usage, 0.0, 100.0),
            SwapTotalBytes = status.UllTotalPageFile,
            SwapUsedBytes = status.UllTotalPageFile > status.UllAvailPageFile
                ? status.UllTotalPageFile - status.UllAvailPageFile
                : 0,
        };
    }

    private static List<DiskMetrics> ReadDisks()
    {
        var result = new List<DiskMetrics>();
        foreach (var drive in DriveInfo.GetDrives())
        {
            if (!drive.IsReady || drive.DriveType != DriveType.Fixed)
            {
                continue;
            }

            try
            {
                ulong total = (ulong)drive.TotalSize;
                ulong free = (ulong)drive.AvailableFreeSpace;
                ulong used = total > free ? total - free : 0;
                double usage = total == 0 ? 0.0 : 100.0 * (double)used / total;

                result.Add(new DiskMetrics
                {
                    Device = drive.Name.TrimEnd('\\'),
                    MountPoint = drive.Name,
                    FsType = drive.DriveFormat,
                    TotalBytes = total,
                    UsedBytes = used,
                    AvailableBytes = free,
                    UsagePercent = Math.Round(usage, 2),
                });
            }
            catch (IOException)
            {
                // 驱动器瞬时不可读：跳过该盘
            }
        }

        return result;
    }
}