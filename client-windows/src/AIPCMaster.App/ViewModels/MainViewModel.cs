// 主视图模型：编排采集→诊断→建议→优化→云端上报全链路。
// 四个页面共享此单一实例（结构化共享状态）。

using AIPCMaster.App.Services;
using AIPCMaster.Core.Advisor;
using AIPCMaster.Core.Collectors;
using AIPCMaster.Core.Diagnosis;
using AIPCMaster.Core.Optimization;

namespace AIPCMaster.App.ViewModels;

public sealed class MainViewModel : ObservableObject
{
    private readonly ISystemCollector _collector;
    private readonly LocalSettings _settings;
    private readonly ApiClient _api;
    private readonly DiagnosticEngine _engine = new();
    private readonly Advisor _advisor = new();
    private readonly OptimizationExecutor _executor = new();
    private readonly AuditLogger _audit = new();

    private readonly Timer _pollTimer;
    private int _pollTick;

    public MainViewModel(ISystemCollector collector, LocalSettings settings, ApiClient api)
    {
        _collector = collector;
        _settings = settings;
        _api = api;

        // 周期采集：正常 5s（PRD §5 硬性）。异常档 500ms 触发后由 RefreshDashboardAsync 加速重采。
        _pollTimer = new Timer(_ => PollTick(), null, TimeSpan.FromSeconds(5), TimeSpan.FromSeconds(5));
    }

    // ---- 观测属性 ----

    private string _statusMessage = "就绪";
    public string StatusMessage
    {
        get => _statusMessage;
        set => Set(ref _statusMessage, value);
    }

    private int _healthScore = 100;
    public int HealthScore
    {
        get => _healthScore;
        set => Set(ref _healthScore, value);
    }

    private string _rootCauseSummary = "尚未诊断";
    public string RootCauseSummary
    {
        get => _rootCauseSummary;
        set => Set(ref _rootCauseSummary, value);
    }

    private System.Collections.ObjectModel.ObservableCollection<Issue> _issues = [];
    public System.Collections.ObjectModel.ObservableCollection<Issue> Issues
    {
        get => _issues;
        set => Set(ref _issues, value);
    }

    private System.Collections.ObjectModel.ObservableCollection<Suggestion> _suggestions = [];
    public System.Collections.ObjectModel.ObservableCollection<Suggestion> Suggestions
    {
        get => _suggestions;
        set => Set(ref _suggestions, value);
    }

    private string _loggedInStatus = "未登录";
    public string LoggedInStatus
    {
        get => _loggedInStatus;
        set => Set(ref _loggedInStatus, value);
    }

    private string _serverStatus = "云端未连接";
    public string ServerStatus
    {
        get => _serverStatus;
        set => Set(ref _serverStatus, value);
    }

    private string _deviceId = "";
    public string DeviceId
    {
        get => _deviceId;
        set => Set(ref _deviceId, value);
    }

    private string _apiBaseUrl = "";
    public string ApiBaseUrl
    {
        get => _apiBaseUrl;
        set => Set(ref _apiBaseUrl, value);
    }

    private string _email = "";
    public string Email
    {
        get => _email;
        set => Set(ref _email, value);
    }

    private string _loginPassword = "";
    public string LoginPassword
    {
        get => _loginPassword;
        set => Set(ref _loginPassword, value);
    }

    // ---- 命令 ----

    public AsyncRelayCommand RunDiagnosisCommand => new(RunDiagnosisAsync);
    public AsyncRelayCommand LoginCommand => new(async () => await LoginAsync());
    public AsyncRelayCommand RegisterCommand => new(async () => await RegisterAsync());
    public AsyncRelayCommand SaveSettingsCommand => new(async () => await SaveSettingsAsync());

    /// 执行优化：高危动作先弹确认框（US-03 执行前确认）。
    public AsyncRelayCommand<Suggestion> ExecuteSuggestionCommand => new(async s =>
    {
        if (s.RequiresConfirm)
        {
            var ok = System.Windows.MessageBox.Show(
                $"「{s.Reason}」\n\n该动作为高风险操作，执行前将创建系统还原点。是否继续？",
                "AIPCMaster - 操作确认",
                System.Windows.MessageBoxButton.YesNo,
                System.Windows.MessageBoxImage.Warning);
            if (ok != System.Windows.MessageBoxResult.Yes)
            {
                StatusMessage = "已取消执行";
                return;
            }
        }

        await ExecuteSuggestionAsync(s); // 忽略返回的消息文本（已写入 StatusMessage）
    });

    // ---- 诊断 ----

    public async Task RefreshDashboardAsync()
    {
        var snap = await Task.Run(() => _collector.CollectFull());
        var report = _engine.Diagnose(snap, Guid.NewGuid().ToString("N"));
        var advice = _advisor.Advise(report);

        HealthScore = report.HealthScore;
        RootCauseSummary = advice.RootCauseSummary;
        Issues = new System.Collections.ObjectModel.ObservableCollection<Issue>(report.Issues);
        Suggestions = new System.Collections.ObjectModel.ObservableCollection<Suggestion>(advice.Suggestions);
        DeviceId = _settings.DeviceId;
        ApiBaseUrl = _settings.ApiBaseUrl;
        Email = _settings.Email ?? "";

        StatusMessage = $"已完成诊断（健康分 {report.HealthScore}，问题 {report.Issues.Count} 项）";

        // 异常 -> 上报云端 + 切 500ms 高频采样
        if (report.HealthScore < 80)
        {
            StatusMessage += "｜系统存在异常，已切换高频采样";
            _pollTimer.Change(TimeSpan.FromMilliseconds(500), TimeSpan.FromMilliseconds(500));
        }
        else
        {
            _pollTimer.Change(TimeSpan.FromSeconds(5), TimeSpan.FromSeconds(5));
        }
    }

    private void PollTick()
    {
        _pollTick++;
        // 周期刷新（保持健康分/建议实时性）；异常档 500ms 刷新
        _ = RefreshDashboardAsync();
    }

    // ---- 认证 ----

    public async Task LoginAsync()
    {
        if (string.IsNullOrWhiteSpace(Email) || string.IsNullOrWhiteSpace(LoginPassword))
        {
            StatusMessage = "请输入邮箱和密码";
            return;
        }

        StatusMessage = "登录中…";
        var env = await _api.LoginAsync(Email.Trim(), LoginPassword);
        if (env.Code != 0 || env.Data is not { } data)
        {
            StatusMessage = $"登录失败: {env.Message}";
            return;
        }

        _api.Tokens = ApiClient.Deserialize<TokenPair>(data.GetProperty("tokens"));
        _settings.Email = Email.Trim();
        _settings.RefreshToken = _api.Tokens?.RefreshToken;
        _settings.Save();

        LoggedInStatus = $"已登录: {Email.Trim()}";
        StatusMessage = "登录成功";
        await SyncDeviceAsync();
    }

    public async Task RegisterAsync()
    {
        if (string.IsNullOrWhiteSpace(Email) || string.IsNullOrWhiteSpace(LoginPassword))
        {
            StatusMessage = "请输入邮箱和密码";
            return;
        }

        StatusMessage = "注册中…";
        var env = await _api.RegisterAsync(Email.Trim(), LoginPassword);
        if (env.Code != 0)
        {
            StatusMessage = $"注册失败: {env.Message}";
            return;
        }

        StatusMessage = "注册成功，已自动登录";
        await LoginAsync();
    }

    // ---- 设备绑定（PRD §4.4） ----

    private async Task SyncDeviceAsync()
    {
        try
        {
            var env = await _api.ListDevicesAsync();
            if (env.Code != 0 || env.Data is not { } data)
            {
                ServerStatus = $"云端: {env.Message}";
                return;
            }

            var json = data.GetRawText();
            if (json.Contains($"\"device_name\":\"{_settings.DeviceName}\"", StringComparison.OrdinalIgnoreCase))
            {
                ServerStatus = "云端: 设备已绑定";
                return;
            }

            var reg = await _api.RegisterDeviceAsync(
                _settings.DeviceName,
                Environment.OSVersion.VersionString,
                typeof(MainViewModel).Assembly.GetName().Version?.ToString() ?? "1.0.0");

            ServerStatus = reg.Code == 0 ? "云端: 设备绑定成功" : $"云端: {reg.Message}";
        }
        catch (Exception e) when (e is HttpRequestException or InvalidOperationException or TaskCanceledException)
        {
            ServerStatus = "云端: 服务不可达";
        }
    }

    // ---- 优化执行（US-03） ----

    public async Task<string> ExecuteSuggestionAsync(Suggestion suggestion)
    {
        // 高危动作先确认（SD §2.3）
        if (suggestion.RequiresConfirm)
        {
            _audit.Record(suggestion.ActionType.ToString().ToLowerInvariant(), "approved",
                $"用户确认执行: {suggestion.Reason}", confirmedByUser: true);
        }

        var result = await Task.Run(() => _executor.Execute(suggestion));
        _audit.Record(
            suggestion.ActionType.ToString().ToLowerInvariant(),
            result.Success ? "executed" : "failed",
            $"{result.Message}｜{string.Join("；", result.RollbackNotes)}",
            confirmedByUser: suggestion.RequiresConfirm,
            restorePointCreated: result.RestorePointCreated);

        StatusMessage = result.Success ? $"✔ {result.Message}" : $"✘ {result.Message}";
        return result.Message;
    }

    // ---- 设置 ----

    public async Task SaveSettingsAsync()
    {
        _settings.ApiBaseUrl = ApiBaseUrl.TrimEnd('/');
        _settings.Save();
        StatusMessage = $"设置已保存（API: {_settings.ApiBaseUrl}）";
        await Task.CompletedTask;
    }

    public void OnNavigatedToSettings()
    {
        DeviceId = _settings.DeviceId;
        ApiBaseUrl = _settings.ApiBaseUrl;
        Email = _settings.Email ?? "";
    }
}