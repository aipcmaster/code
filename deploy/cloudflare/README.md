# aipcmaster.com 缓存与性能配置（Cloudflare → GitHub Pages）

适用站点：`https://aipcmaster.com`（GitHub Pages 源站 + Cloudflare 代理）

> **套餐限制（踩过的坑）**：`matches`（正则）算符需要 **Business / Enterprise** 套餐，
> Free / Pro 使用会整条规则被拒（HTTP 400 `not entitled: the use of operator Matches`）。
> `starts_with` / `ends_with` 在自定义规则里**根本不支持**。
> 因此本配置只用 `eq` / `contains` / `in` —— 全套餐可用。
> 另：`edge_ttl.mode` 的正确枚举是 `override_origin`（不是 `override`）。

## 0. 最快路径：一条命令
不想点面板的话，用脚本。它会自动找 zone、**合并**现有规则（幂等）、应用并核验：

```bash
# 先在面板建一个 token（My Profile → API Tokens → Create Token → Custom token）
#   权限：Zone → Cache Rules → Edit   和   Zone → Zone → Read
#   范围：Include → Specific zone → aipcmaster.com

# 0) 不需要 token 的两项自检（建议先跑，确认脚本行为符合预期）
python3 deploy/cloudflare/apply.py --self-test       # 12 条真实路径的路由断言
python3 deploy/cloudflare/apply.py --print-payload   # 打印将要提交的 JSON

# 1) 预览（会连 Cloudflare 读现有规则，但不改动任何东西）
CF_API_TOKEN=xxxxx python3 deploy/cloudflare/apply.py --zone aipcmaster.com --dry-run

# 2) 应用
CF_API_TOKEN=xxxxx python3 deploy/cloudflare/apply.py --zone aipcmaster.com

# 3) 只核验线上响应头（不需要 token）
python3 deploy/cloudflare/apply.py --verify-only
```

token 只从环境变量读取（不写盘、不回显、不作为命令行参数）。下面的手工步骤仅作参考/审计用途。

## 1. 现状（2026-09-14 实测）

| 资源 | 源站 Cache-Control | Cloudflare 状态 | 含义 |
| - | - | - | - |
| `/`、`/index.html`、`/sitemap.xml` | `max-age=600` | `cf-cache-status: DYNAMIC` | **边缘完全不缓存**，每次访问都回源 GitHub Pages |
| `/og-pricing.png`、`/app.js` | `max-age=14400`（4 小时） | `cf-cache-status: EXPIRED` | 缓存过但已过期，回源 |
| 其他 | `Server: cloudflare`、`alt-svc: h3` | — | 已启用 HTTP/3 |

**两个问题：**

1. **HTML 是 `DYNAMIC`** —— Cloudflare 默认不缓存 HTML。每次页面访问都要跨洋回源 GitHub Pages（首次字节 TTFB 因此偏高）。
2. **静态资源只有 4 小时** —— `og-*.png`（22 张）、`app.js`、`favicon.svg` 反复回源，社交爬虫每次抓取分享图都打源站。

**为什么 HTML 缓存要短**：本站 CSS 已内联进 HTML，HTML 过期 = 样式过期。所以 HTML 必须短 TTL，静态资源可以长 TTL。

---

## 2. 目标

| 类别 | 边缘 TTL | 浏览器 TTL | 理由 |
| - | - | - | - |
| 静态资源（og 图 / app.js / favicon） | 1 年 | 1 年 | 内容极少变；改动时靠"部署后清理" |
| HTML | 5 分钟 | 1 分钟 | 内联了 CSS，必须能快速更新 |
| sitemap / robots / llms / humans | 1 小时 | 1 小时 | 爬虫读取，无需实时 |

---

## 3. 配置步骤

登录 Cloudflare → 选择 `aipcmaster.com` 区域。

### 3.1 三条 Cache Rule

**Caching → Cache Rules → Create rule**，依次建三条：

#### 规则 A —— 静态资源长缓存

- **Rule name**: `Static assets — long cache`
- **When incoming requests match**（Edit expression，粘贴）:
  ```
  (http.request.uri.path eq "/app.js" or http.request.uri.path eq "/favicon.svg" or (http.request.uri.path contains "/og-" and http.request.uri.path contains ".png"))
  ```
- **Then**:
  | 设置 | 值 |
  | - | - |
  | Cache eligibility | Eligible for cache |
  | Edge TTL | Override origin → **1 year** |
  | Browser TTL | Override origin → **1 year** |

#### 规则 B —— HTML 短缓存

- **Rule name**: `HTML — short cache`
- **When**:
  ```
  (http.request.uri.path eq "/" or http.request.uri.path contains ".html")
  ```
- **Then**:
  | 设置 | 值 |
  | - | - |
  | Cache eligibility | Eligible for cache |
  | Edge TTL | Override origin → **5 minutes** |
  | Browser TTL | Override origin → **1 minute** |

#### 规则 C —— 爬虫文件中缓存

- **Rule name**: `Crawler files — medium cache`
- **When**:
  ```
  (http.request.uri.path in {"/sitemap.xml" "/robots.txt" "/llms.txt" "/humans.txt"})
  ```
- **Then**:
  | 设置 | 值 |
  | - | - |
  | Cache eligibility | Eligible for cache |
  | Edge TTL | Override origin → **1 hour** |
  | Browser TTL | Override origin → **1 hour** |

> **顺序很重要**：Cache Rules 自上而下匹配。把规则 A 放在最上（静态资源优先），B 其次，C 最后。

### 3.2 优化开关

**Speed → Optimization**：

| 开关 | 设为 | 说明 |
| - | - | - |
| Brotli | **On** | 比 gzip 再小 ~15% |
| Early Hints | **On** | HTML 返回 103，提前预取 `app.js` |
| HTTP/3 (with QUIC) | **On**（实测已开） | — |
| 0-RTT Connection Resumption | On | 回访更快 |

**Caching → Tiered Cache**：**On**（免费版可用）。上层 PoP 缓存未命中时先问上层，减少回源。

### 3.3 部署后清理缓存（关键）

规则 A 给了 1 年 TTL，**每次部署后必须清理**，否则用户拿到旧的 `app.js` / OG 图。

在推送 `aipcmaster/web` 之后执行其一：

**方式一：面板**
Caching → Configuration → Purge Cache → **Purge Everything**

**方式二：API（推荐，可脚本化）**
```bash
CF_TOKEN=...            # 权限：Zone → Cache Purge → Purge
CF_ZONE=...             # aipcmaster.com 的 Zone ID
curl -X POST "https://api.cloudflare.com/client/v4/zones/$CF_ZONE/purge_cache" \
  -H "Authorization: Bearer $CF_TOKEN" \
  -H "Content-Type: application/json" \
  --data '{"files":[
    "https://aipcmaster.com/app.js",
    "https://aipcmaster.com/favicon.svg"
  ]}'
```
OG 图数量多，直接 `--data '{"purge_everything":true}'` 更省事（本站流量小，全量清理无副作用）。

---

## 4. 验证

```bash
# HTML：应看到 cf-cache-status: MISS（首次）→ HIT（第二次）
curl -sI https://aipcmaster.com/ | grep -Ei "cache-control|cf-cache-status|age"

# 静态资源：应看到 max-age=31536000 且 HIT
curl -sI https://aipcmaster.com/app.js | grep -Ei "cache-control|cf-cache-status|age"

# 压缩：浏览器会带 Accept-Encoding，确认返回 br 或 gzip
curl -sI -H "Accept-Encoding: br,gzip" https://aipcmaster.com/ | grep -i content-encoding
```

期望：
- HTML：`cache-control: max-age=60`（浏览器）+ 边缘 `cf-cache-status: HIT`，`age` 递增
- `app.js`：`cache-control: max-age=31536000`，`cf-cache-status: HIT`

---

## 5. 可选进阶：指纹化文件名（彻底 immutable）

当前 `app.js`、`og-*.png` 文件名不含内容哈希，所以只能用"长 TTL + 部署后清理"。
若要做到**永不需要清理**：把资源改为内容哈希命名（如 `app.<hash>.js`），
并把 TTL 设为 `immutable`。

这需要改 `web-src/generate.py`（生成时计算哈希并重写引用）。代价是 HTML 里的引用会随内容变化，
但 HTML 本身是短 TTL，所以没问题。**当前规模不必做**，除非静态资源开始频繁变动。

---

## 6. 回滚

删掉三条 Cache Rule 即可回到 Cloudflare 默认行为（HTML `DYNAMIC`、静态资源跟随源站 `max-age=14400`）。
清理缓存不会影响站点内容。
