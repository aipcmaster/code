# aipcmaster.com 缓存配置（Cloudflare Free → GitHub Pages）

适用站点：`https://aipcmaster.com`（GitHub Pages 源站 + Cloudflare 代理）

## 0. 最快路径：一条命令

```bash
# 建 token（My Profile → API Tokens → Create Token → Custom token）
#   权限：Zone → Cache Rules → Edit   和   Zone → Zone → Read
#   范围：Include → Specific zone → aipcmaster.com

# 不需要 token 的自检
python3 deploy/cloudflare/apply.py --self-test
python3 deploy/cloudflare/apply.py --print-payload

# 预览 → 应用
CF_API_TOKEN=xxxxx python3 deploy/cloudflare/apply.py --zone aipcmaster.com --dry-run
CF_API_TOKEN=xxxxx python3 deploy/cloudflare/apply.py --zone aipcmaster.com

# 核验（不需要 token）
python3 deploy/cloudflare/apply.py --verify-only
```

token 只从环境变量读取（不写盘、不回显、不作为命令行参数）。

---

## 1. Free 套餐的三条硬限制（都踩过）

| 限制 | Free | 影响 |
| - | - | - |
| `matches`（正则）算符 | ❌ 需 Business/Enterprise | 表达式只能用 `eq` / `contains` / `in` |
| `starts_with` / `ends_with` | ❌ 自定义规则根本不支持 | 同上 |
| **最小 Edge Cache TTL** | **2 小时** | 低于 7200s 会被 API 拒绝 |
| 最小 Browser TTL | 1 秒 | — |
| Cache Rules 条数上限 | 10 | 本配置用 2 条 |

另：`edge_ttl.mode` 的正确枚举是 `override_origin`（不是 `override`）。

### 为什么默认不缓存 HTML

Free 的最小 Edge TTL 是 2 小时。把 HTML 缓存 2 小时意味着**内容更新后最多 2 小时用户才看到**，
比现在（不缓存、每次回源）更糟。所以默认只做两件事：

| 规则 | 内容 | 边缘 TTL | 浏览器 TTL |
| - | - | - | - |
| static assets | `app.js`、`favicon.svg`、22 张 `og-*.png` | 1 年 | 1 年 |
| crawler files | `sitemap.xml`、`robots.txt`、`llms.txt`、`humans.txt` | 2 小时（Free 下限） | 2 小时 |

HTML 保持 Cloudflare 默认（`cf-cache-status: DYNAMIC`，不缓存边缘）—— 这是 Free 套餐下的正确取舍。
GitHub Pages 本身就在 CDN 后面，回源不算慢。

如果你接受"部署后手动清缓存"这个操作，可以加上 HTML 缓存：

```bash
CF_API_TOKEN=xxxxx python3 deploy/cloudflare/apply.py --zone aipcmaster.com --cache-html
```

它会把 HTML 也缓存到边缘，TTL 为 Free 下限 2 小时。**代价**：每次部署后必须 Purge，
否则最多 2 小时用户看到旧页面。

---

## 2. 现状（2026-09-14 实测，未配置前）

| 资源 | 源站 Cache-Control | Cloudflare 状态 |
| - | - | - |
| HTML（`/`、`/index.html`） | `max-age=600` | `DYNAMIC`（边缘完全不缓存） |
| `/og-*.png`、`/app.js` | `max-age=14400`（4 小时） | `EXPIRED`（反复回源） |
| `/sitemap.xml` | `max-age=600` | `DYNAMIC` |

问题：22 张 OG 图被社交爬虫反复抓取打源站；`app.js` 只有 4 小时缓存。

---

## 3. 脚本会做什么

1. 用 token 查 zone ID
2. 读取该 zone 现有的 cache 规则
3. **只替换带 `[aipcmaster]` 标记的规则**，你原有的其他规则原样保留
4. 我们的规则排在前面（匹配自上而下）
5. `PUT` 到 `http_request_cache_settings` phase 的 entrypoint
6. 回读并打印结果，然后核验线上响应头

脚本只动 cache settings 这一个 phase —— **不影响** Redirect Rules / WAF / Transform Rules。

---

## 4. 验证

```bash
python3 deploy/cloudflare/apply.py --verify-only
```

期望（配置后）：

```
静态资源   max-age=31536000    cf-cache-status=HIT   age 递增
HTML      max-age=600          cf-cache-status=DYNAMIC
```

第一次 `MISS` 正常（缓存是访问时才建立的），隔几秒再跑一次就是 `HIT`。

---

## 5. 部署后清理缓存（用 --cache-html 时才必须）

默认配置下，静态资源是 1 年 TTL，所以**改了 `app.js` 或 OG 图之后需要清理**：

- 面板：Caching → Configuration → **Purge Everything**
- 或 API：`POST /zones/{zone_id}/purge_cache`，body `{"purge_everything": true}`

HTML 不缓存边缘，所以正文更新不需要清理。

---

## 6. 进阶：指纹化文件名（彻底免清理）

把 `app.js`、`og-*.png` 改成内容哈希命名（`app.<hash>.js`），TTL 设为 immutable，
就再也不需要 Purge。需要改 `web-src/generate.py`。当前规模不必做。

---

## 7. 回滚

删掉面板里的 `[aipcmaster]` 规则（或重跑脚本前把 `build_rules` 改空）即可回到 Cloudflare 默认行为。
清理缓存不会影响站点内容。
