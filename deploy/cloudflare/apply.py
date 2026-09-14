#!/usr/bin/env python3
"""Apply the aipcmaster.com cache rules to Cloudflare in one command.

Does the dashboard work for you: finds the zone, reads whatever cache rules
already exist, merges ours in (idempotent — safe to re-run), applies, and
verifies the live response headers.

Setup (once, in the Cloudflare dashboard):
  My Profile → API Tokens → Create Token → Custom token
    Permissions:
      Zone → Cache Rules → Edit
      Zone → Zone      → Read
    Zone Resources: Include → Specific zone → aipcmaster.com
  Copy the token. It is shown once.

Run:
  CF_API_TOKEN=xxxxx python3 deploy/cloudflare/apply.py --zone aipcmaster.com
  CF_API_TOKEN=xxxxx python3 deploy/cloudflare/apply.py --zone aipcmaster.com --dry-run
  python3 deploy/cloudflare/apply.py --verify-only          # no token needed

The token is read from the environment only. It is never written to disk,
never echoed, and never passed as a command-line argument.
"""
import argparse
import json
import os
import sys
import urllib.error
import urllib.request

API = "https://api.cloudflare.com/client/v4"
PHASE = "http_request_cache_settings"
TAG = "[aipcmaster]"  # description prefix used to find/replace our own rules

# Rules in match order. Static assets first so they win over the HTML rule.
RULES = [
    {
        "description": f"{TAG} static assets — long cache",
        "expression": r'(http.request.uri.path matches "^/(og-.*\.png|favicon\.svg|app\.js)$")',
        "action": "set_cache_settings",
        "action_parameters": {
            "cache": True,
            "edge_ttl": {"mode": "override", "default": 31536000},   # 1 year
            "browser_ttl": {"mode": "override", "default": 31536000},
        },
    },
    {
        "description": f"{TAG} HTML — short cache",
        "expression": r'(http.request.uri.path eq "/" or http.request.uri.path matches "\.html$")',
        "action": "set_cache_settings",
        "action_parameters": {
            "cache": True,
            "edge_ttl": {"mode": "override", "default": 300},        # 5 minutes
            "browser_ttl": {"mode": "override", "default": 60},      # 1 minute
        },
    },
    {
        "description": f"{TAG} crawler files — medium cache",
        "expression": '(http.request.uri.path in {"/sitemap.xml" "/robots.txt" "/llms.txt" "/humans.txt"})',
        "action": "set_cache_settings",
        "action_parameters": {
            "cache": True,
            "edge_ttl": {"mode": "override", "default": 3600},       # 1 hour
            "browser_ttl": {"mode": "override", "default": 3600},
        },
    },
]


def request(method, path, token, body=None):
    data = json.dumps(body).encode() if body is not None else None
    req = urllib.request.Request(f"{API}{path}", data=data, method=method)
    req.add_header("Authorization", f"Bearer {token}")
    req.add_header("Content-Type", "application/json")
    try:
        with urllib.request.urlopen(req, timeout=30) as r:
            return json.loads(r.read())
    except urllib.error.HTTPError as e:
        payload = e.read().decode("utf-8", "replace")
        try:
            errs = json.loads(payload).get("errors", [])
            detail = "; ".join(f"{x.get('code')}: {x.get('message')}" for x in errs) or payload[:300]
        except json.JSONDecodeError:
            detail = payload[:300]
        hint = ""
        if e.code == 403:
            hint = "\n  → token 缺少权限：需要 Zone → Cache Rules → Edit 和 Zone → Zone → Read"
        elif e.code == 404:
            hint = "\n  → zone 名或 zone_id 不对"
        raise SystemExit(f"HTTP {e.code} {method} {path}\n  {detail}{hint}") from None


def find_zone(token, zone_name, zone_id):
    if zone_id:
        return zone_id
    res = request("GET", f"/zones?name={zone_name}", token)
    result = res.get("result") or []
    if not result:
        raise SystemExit(f"找不到 zone: {zone_name}（确认域名拼写，且 token 有 Zone → Zone → Read）")
    return result[0]["id"]


def get_existing(token, zid):
    """Return the current entrypoint rules, or [] if the phase has none yet."""
    try:
        res = request("GET", f"/zones/{zid}/rulesets/phases/{PHASE}/entrypoint", token)
    except SystemExit as e:
        if "HTTP 404" in str(e):
            return []
        raise
    return res.get("result", {}).get("rules") or []


def main():
    ap = argparse.ArgumentParser(description="Apply aipcmaster.com Cloudflare cache rules")
    ap.add_argument("--zone", default="aipcmaster.com", help="zone name (default: aipcmaster.com)")
    ap.add_argument("--zone-id", help="zone id, skips the lookup")
    ap.add_argument("--dry-run", action="store_true", help="print the payload, change nothing")
    ap.add_argument("--verify-only", action="store_true", help="only check live headers")
    args = ap.parse_args()

    if args.verify_only:
        verify()
        return

    token = os.environ.get("CF_API_TOKEN")
    if not token:
        raise SystemExit(
            "未设置 CF_API_TOKEN。\n"
            "  CF_API_TOKEN=xxxxx python3 deploy/cloudflare/apply.py --zone aipcmaster.com\n"
            "（token 只从环境变量读取，避免留在 shell 历史里）"
        )

    zid = find_zone(token, args.zone, args.zone_id)
    print(f"zone: {args.zone}  ({zid})")

    existing = get_existing(token, zid)
    kept = [r for r in existing if not str(r.get("description", "")).startswith(TAG)]
    ours = [r for r in existing if str(r.get("description", "")).startswith(TAG)]
    print(f"现有 cache 规则 {len(existing)} 条：保留 {len(kept)} 条，替换本工具的 {len(ours)} 条")

    payload = {"rules": RULES + kept}  # ours first: match order is top-down

    if args.dry_run:
        print("\n--- dry run, 不会提交 ---")
        print(json.dumps(payload, ensure_ascii=False, indent=2))
        return

    request("PUT", f"/zones/{zid}/rulesets/phases/{PHASE}/entrypoint", token, payload)

    applied = get_existing(token, zid)
    print(f"\n✅ 已应用 {len(applied)} 条规则：")
    for r in applied:
        ttl = r.get("action_parameters", {}).get("edge_ttl", {})
        print(f"   - {r.get('description')}  edge_ttl={ttl.get('default')}s")
    print("\n注意：长缓存已生效。之后每次部署 aipcmaster/web 后需要清理缓存")
    print("（Cloudflare 面板 → Caching → Configuration → Purge Everything）。")

    print()
    verify()


def verify():
    import time

    print("实时响应头核验（首次 MISS → 再访问 HIT）")
    for path, label in (("", "HTML"), ("app.js", "静态资源")):
        url = f"https://aipcmaster.com/{path}"
        try:
            req = urllib.request.Request(url, headers={"User-Agent": "Mozilla/5.0"})
            with urllib.request.urlopen(req, timeout=25) as r:
                h = r.headers
            print(f"  {label:8} {h.get('Cache-Control','?'):<26} "
                  f"cf-cache-status={h.get('cf-cache-status','?'):<8} age={h.get('Age','0')}")
        except Exception as e:
            print(f"  {label:8} 检查失败: {type(e).__name__} {e}")
    print("  （边缘刚建立缓存时可能先 MISS，稍等几秒再跑一次即 HIT）")


if __name__ == "__main__":
    main()
