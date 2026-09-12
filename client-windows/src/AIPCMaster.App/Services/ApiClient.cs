// 云端 REST 客户端 —— 对接 aipcmaster-server（SD §3.2，统一响应契约 §4.1）。
//
// 统一响应：{ code, message, data, request_id }
//   0 = 成功；40001/40101/40301/40401/40901/42901/50001/60001/60002（SD §4.4）
//
// 零第三方依赖：System.Net.Http + System.Text.Json。

using System.Net.Http.Headers;
using System.Text;
using System.Text.Json;

namespace AIPCMaster.App.Services;

/// 统一响应信封（SD §4.1）。
public sealed class ApiEnvelope
{
    public int Code { get; set; }
    public string Message { get; set; } = "";
    public JsonElement? Data { get; set; }

    [System.Text.Json.Serialization.JsonPropertyName("request_id")]
    public string RequestId { get; set; } = "";
}

/// 认证令牌对（与 server TokenPair 对齐）。
public sealed class TokenPair
{
    [System.Text.Json.Serialization.JsonPropertyName("access_token")]
    public string AccessToken { get; set; } = "";

    [System.Text.Json.Serialization.JsonPropertyName("refresh_token")]
    public string RefreshToken { get; set; } = "";

    [System.Text.Json.Serialization.JsonPropertyName("token_type")]
    public string TokenType { get; set; } = "Bearer";

    [System.Text.Json.Serialization.JsonPropertyName("expires_in")]
    public long ExpiresIn { get; set; }
}

/// 统一 API 客户端（JWT 自动附加 + 刷新）。
public sealed class ApiClient : IDisposable
{
    private readonly HttpClient _http;
    private readonly SemaphoreSlim _refreshLock = new(1, 1);

    public string BaseUrl { get; }
    public TokenPair? Tokens { get; set; }
    public string? Email { get; set; }

    public ApiClient(string baseUrl)
    {
        BaseUrl = baseUrl.TrimEnd('/');
        _http = new HttpClient { BaseAddress = new Uri(BaseUrl) };
        _http.Timeout = TimeSpan.FromSeconds(30);
        _http.DefaultRequestHeaders.Accept.Add(new MediaTypeWithQualityHeaderValue("application/json"));
    }

    // ---- 核心请求 ----

    private async Task<ApiEnvelope> PostAsync(string path, object? body, bool withAuth, CancellationToken ct = default)
    {
        var req = new HttpRequestMessage(HttpMethod.Post, path);
        if (withAuth)
        {
            AttachAuth(req);
        }

        if (body is not null)
        {
            req.Content = new StringContent(
                JsonSerializer.Serialize(body),
                Encoding.UTF8,
                "application/json");
        }

        return await SendWithRefreshAsync(req, withAuth, ct).ConfigureAwait(false);
    }

    private async Task<ApiEnvelope> GetAsync(string path, bool withAuth, CancellationToken ct = default)
    {
        var req = new HttpRequestMessage(HttpMethod.Get, path);
        if (withAuth)
        {
            AttachAuth(req);
        }

        return await SendWithRefreshAsync(req, withAuth, ct).ConfigureAwait(false);
    }

    private void AttachAuth(HttpRequestMessage req)
    {
        if (Tokens?.AccessToken is { Length: > 0 } token)
        {
            req.Headers.Authorization = new AuthenticationHeaderValue("Bearer", token);
        }
    }

    /// 401 时自动用 refresh token 换新 access token 并重试一次。
    private async Task<ApiEnvelope> SendWithRefreshAsync(HttpRequestMessage req, bool withAuth, CancellationToken ct)
    {
        var resp = await _http.SendAsync(req, ct).ConfigureAwait(false);
        var env = await ParseAsync(resp, ct).ConfigureAwait(false);

        if (withAuth && env.Code == 40101 && Tokens?.RefreshToken is { Length: > 0 })
        {
            await _refreshLock.WaitAsync(ct);
            try
            {
                var refreshed = await DoRefreshAsync(ct).ConfigureAwait(false);
                if (refreshed)
                {
                    // 重试原请求（新 access token）
                    var retry = new HttpRequestMessage(req.Method, req.RequestUri);
                    if (req.Content is not null)
                    {
                        retry.Content = req.Content;
                    }

                    AttachAuth(retry);
                    var retryResp = await _http.SendAsync(retry, ct).ConfigureAwait(false);
                    return await ParseAsync(retryResp, ct).ConfigureAwait(false);
                }
            }
            finally
            {
                _refreshLock.Release();
            }
        }

        return env;
    }

    private async Task<bool> DoRefreshAsync(CancellationToken ct)
    {
        try
        {
            var env = await PostAsync("/api/v1/auth/refresh", new { refresh_token = Tokens!.RefreshToken }, withAuth: false, ct).ConfigureAwait(false);
            if (env.Code != 0 || env.Data is not { } data)
            {
                Tokens = null;
                return false;
            }

            Tokens = Deserialize<TokenPair>(data.GetProperty("tokens"));
            return true;
        }
        catch (Exception)
        {
            Tokens = null;
            return false;
        }
    }

    private static async Task<ApiEnvelope> ParseAsync(HttpResponseMessage resp, CancellationToken ct)
    {
        var json = await resp.Content.ReadAsStringAsync(ct).ConfigureAwait(false);
        try
        {
            var env = JsonSerializer.Deserialize<ApiEnvelope>(json, JsonOpts) ?? new ApiEnvelope();
            if (env.Code == 0 && !resp.IsSuccessStatusCode)
            {
                // 服务端非 2xx 但消息可解析：透传 code/message
            }

            return env;
        }
        catch (JsonException)
        {
            return new ApiEnvelope
            {
                Code = (int)resp.StatusCode,
                Message = $"HTTP {(int)resp.StatusCode}: {json[..Math.Min(json.Length, 200)]}",
            };
        }
    }

    // ---- 认证 API ----

    public Task<ApiEnvelope> RegisterAsync(string email, string password, string? displayName = null, CancellationToken ct = default)
        => PostAsync("/api/v1/auth/register", new { email, password, display_name = displayName }, withAuth: false, ct);

    public Task<ApiEnvelope> LoginAsync(string email, string password, CancellationToken ct = default)
        => PostAsync("/api/v1/auth/login", new { email, password }, withAuth: false, ct);

    public Task<ApiEnvelope> MeAsync(CancellationToken ct = default)
        => GetAsync("/api/v1/users/me", withAuth: true, ct);

    // ---- 设备 API ----

    public Task<ApiEnvelope> ListDevicesAsync(CancellationToken ct = default)
        => GetAsync("/api/v1/devices", withAuth: true, ct);

    public Task<ApiEnvelope> RegisterDeviceAsync(string deviceName, string os, string version, CancellationToken ct = default)
        => PostAsync("/api/v1/devices/register", new { device_name = deviceName, os, version }, withAuth: true, ct);

    // ---- 诊断 API ----

    public Task<ApiEnvelope> CreateDiagnosticSessionAsync(string deviceId, string? triggerType = null, CancellationToken ct = default)
        => PostAsync("/api/v1/diagnostics/sessions", new
        {
            device_id = deviceId,
            trigger_type = triggerType ?? "manual",
        }, withAuth: true, ct);

    // ---- 优化 API（状态机 suggested → executed） ----

    public Task<ApiEnvelope> ProposeOptimizationAsync(
        string deviceId, string? issueId, string actionType, string riskLevel,
        bool requiresConfirm, JsonElement parameters, CancellationToken ct = default)
        => PostAsync("/api/v1/optimizations/actions", new
        {
            device_id = deviceId,
            issue_id = issueId,
            action_type = actionType,
            risk_level = riskLevel,
            requires_confirm = requiresConfirm,
            parameters,
        }, withAuth: true, ct);

    public Task<ApiEnvelope> ExecuteOptimizationAsync(string actionId, CancellationToken ct = default)
        => PostAsync($"/api/v1/optimizations/actions/{actionId}/execute", null, withAuth: true, ct);

    // ---- 订阅 ----

    public Task<ApiEnvelope> CurrentSubscriptionAsync(CancellationToken ct = default)
        => GetAsync("/api/v1/subscriptions/current", withAuth: true, ct);

    // ---- 工具 ----

    private static readonly JsonSerializerOptions JsonOpts = new()
    {
        PropertyNameCaseInsensitive = true,
        PropertyNamingPolicy = JsonNamingPolicy.CamelCase,
    };

    public static T? Deserialize<T>(JsonElement el) =>
        el.Deserialize<T>(JsonOpts);

    public static T? Deserialize<T>(string json) =>
        JsonSerializer.Deserialize<T>(json, JsonOpts);

    public void Dispose() => _http.Dispose();
}