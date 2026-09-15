# Windows 客户端构建与自检交接单

## 前提条件

| 项 | 要求 |
|---|---|
| 系统 | Windows 10/11（物理机或 VM 均可） |
| SDK | **.NET 8 SDK**（`dotnet --version` 显示 `8.x.x` 即可） |
| 网络 | 公网可达 `https://github.com`（克隆公开仓库） |
| 显示 | 运行 WPF App 需图形桌面（VM 带 GUI 即可） |

### 检查/安装 .NET 8 SDK

```powershell
dotnet --version
# 若未安装或不是 8.x，执行：
# Windows（推荐）
winget install Microsoft.DotNet.SDK.8
# 或下载离线安装包：https://dotnet.microsoft.com/download/dotnet/8.0
```

## 克隆 & 构建

```powershell
git clone https://github.com/aipcmaster/code.git aipcmaster-code
cd aipcmaster-code/client-windows

# 恢复（零 NuGet，秒级完成）
dotnet restore src/AIPCMaster.App/AIPCMaster.App.csproj

# 构建 Release
dotnet build src/AIPCMaster.App/AIPCMaster.App.csproj -c Release
```

> **预期**：构建成功，0 错误，0 NuGet 下载。
> 若有编译报错 → 直接把完整错误输出发回给我（项目负责人），我来修。

## Step 1：运行 ConsoleSelfCheck（免 GUI，命令行执行）

```powershell
dotnet run --project tests/ConsoleSelfCheck/ConsoleSelfCheck.csproj -c Release
```

### 预期输出

所有 Check 均为 **✓**（无 FAIL），类似：

```
采集：✓
诊断：✓（健康分 xx，问题 n 项）
建议：✓
  风险排序 ✓
  去重合并 ✓
  摘要 ✓
  与诊断匹配 ✓
  去重后顺序 ✓
健康分算法：✓
  空问题=100 ✓
  单Critical=60 ✓
  High+Medium=60 ✓
  前3Low取扣=85 ✓
  封底=20 ✓
自检完成: 0 失败
```

**若出现 FAIL** → 把完整输出发回，含具体是哪个 Check 失败及原因。

## Step 2：运行 WPF 应用（需 GUI）

```powershell
dotnet run --project src/AIPCMaster.App/AIPCMaster.App.csproj -c Release
```

### 检查项（每个截图发回）

1. **仪表盘页**：顶部健康分数值是否 20-100，内存/磁盘/负载卡片有数值（不是 0 或 NaN）
2. **诊断页**：每 5 秒自动刷新（点"立即采集"），问题列表按严重度排序
3. **优化页**：有建议条目；点"执行"后弹出确认框；确认后显示"已执行"
4. **设置页**：采样频率选择生效（切换 5s/500ms，回到诊断页观察刷新速度变化）

### 常见问题速查

| 现象 | 处理 |
|---|---|
| 窗口启动后白屏/崩溃 | 截图发回，附 WPF 崩溃日志（Windows 事件查看器 → 应用程序日志） |
| 诊断页数值全 0 | 截图发回（可能 P/Invoke 在 VM 环境权限不足） |
| 内存/磁盘显示 0% | 截图发回（可能是 VM 虚拟磁盘挂载路径不常见） |
| 优化执行报错 | 截图 + 完整错误消息发回 |

## 汇报格式（直接复制填写）

````
## 构建结果
- .NET SDK 版本：`dotnet --version` 输出
- 构建状态：成功 / 失败（贴错误）
- 运行环境：物理机 / Hyper-V VM / VMware VM（注明）

## ConsoleSelfCheck 结果
（粘贴完整输出，或截图）

## WPF 应用检查
- 仪表盘：健康分=xx / 内存=xx% / 磁盘=xx% / 负载=xx
- 诊断页：能否自动刷新？问题列表是否正常？
- 优化页：有无建议条目？执行确认框是否弹出？
- 设置页：采样频率切换是否生效？
- 整体：截图附上（仪表盘 + 诊断 + 优化 各一张）

## 异常/备注
（有任何不符合预期的，描述 + 截图）
````

## 技术备注（供 opencode 阅读）

- **零 NuGet 依赖**：`dotnet restore` 无网络请求，离线环境亦可构建
- **target framework**：Core → `net8.0`；App(WPF) → `net8.0-windows`；SelfCheck → `net8.0`
- **数据采集路径**（P/Invoke，Windows API）：
  - CPU：`GetSystemTimes` 双采样差分（两次调用间隔 500ms）
  - 内存：`GlobalMemoryStatusEx`
  - 磁盘：`System.IO.DriveInfo`
  - 负载：无对应 API（健康分固定 80，规则不触发，**符合预期**）
  - 温度：无 API（固定 36°C，规则跳过，**符合预期**）
- **优化执行器**：`OptimizationExecutor.CleanMemory` 调用内核 `EmptyWorkingSet`；`CleanDisk` 删除 `Path.GetTempPath()` 下超过 30 天的文件；`RestorePoint` 调用 `SRSetRestorePoint`（需管理员权限，VM 里大概率会 403 → **是预期行为**，不影响评分，修复会降级处理）
- **审计日志**：执行后写 `%LOCALAPPDATA%\AIPCMaster\logs\audit-YYYY-MM-DD.jsonl`

---

**把以上汇报内容发回给我，我根据结果决定下一步（可能修 bug、调参、或确认通过）。**
