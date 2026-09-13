// 本地设置 —— %LOCALAPPDATA%/AIPCMaster/settings.json。
// 保存：设备唯一标识、API 地址、登录邮箱、（可选）刷新令牌。

using System.Text.Json;

namespace AIPCMaster.App.Services;

public sealed class LocalSettings
{
    private static readonly string Dir = Path.Combine(
        Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData),
        "AIPCMaster");

    private static readonly string FilePath = Path.Combine(Dir, "settings.json");

    /// 设备唯一标识（PRD §4.4：客户端首次启动生成唯一标识；无需登录即可生成）。
    public string DeviceId { get; set; } = "";

    public string ApiBaseUrl { get; set; } = "http://127.0.0.1:8787";

    public string? Email { get; set; }

    /// 刷新令牌（换取新 access token 用；access token 本身不落盘）。
    ///
    /// 安全说明（CSO 审计 M5）：当前以明文 JSON 存于用户级目录
    /// `%LOCALAPPDATA%/AIPCMaster/settings.json`（该目录默认仅当前用户可读）。
    /// 生产建议改用 Windows DPAPI（ProtectedData.Protect）加密后再落盘——
    /// 需要 NuGet 包 System.Security.Cryptography.ProtectedData，离线环境暂不可用。
    /// 缓解：刷新令牌有效期有限；服务端已支持在刷新时重新读取权限。
    public string? RefreshToken { get; set; }

    /// 设备名（主机名）。
    public string DeviceName { get; set; } = "";

    private static readonly JsonSerializerOptions JsonOpts = new()
    {
        WriteIndented = true,
        PropertyNamingPolicy = JsonNamingPolicy.CamelCase,
    };

    public static LocalSettings Load()
    {
        try
        {
            if (File.Exists(FilePath))
            {
                var s = JsonSerializer.Deserialize<LocalSettings>(File.ReadAllText(FilePath), JsonOpts);
                if (s is not null)
                {
                    s.EnsureDefaults();
                    return s;
                }
            }
        }
        catch (Exception e) when (e is IOException or UnauthorizedAccessException or JsonException)
        {
            // 配置损坏则回退默认
        }

        var fresh = new LocalSettings();
        fresh.EnsureDefaults();
        return fresh;
    }

    private void EnsureDefaults()
    {
        if (string.IsNullOrEmpty(DeviceId))
        {
            DeviceId = Guid.NewGuid().ToString("N");
        }

        if (string.IsNullOrEmpty(DeviceName))
        {
            DeviceName = Environment.MachineName;
        }
    }

    public void Save()
    {
        try
        {
            Directory.CreateDirectory(Dir);
            File.WriteAllText(FilePath, JsonSerializer.Serialize(this, JsonOpts));
        }
        catch (Exception e) when (e is IOException or UnauthorizedAccessException)
        {
            // 存档失败不影响当前会话
        }
    }
}