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

# TTL mode enums. Cloudflare rejects anything else with HTTP 400
# "unknown variant for set_cache_settings_edge_type" — note it is
# `override_origin`, NOT `override`.
TTL_MODES = {"respect_origin", "override_origin", "bypass_by_default", "bypass"}

# Operators that work on every plan. `matches` (regex) needs Business/Enterprise,
# and `starts_with`/`ends_with` are not supported in custom rules at all — using
# either gets the whole rule rejected with HTTP 400. So: eq / contains / in only.
ALLOWED_OPERATORS = (" eq ", " contains ", " in ", " and ", " or ", "not ")
FORBIDDEN_OPERATORS = ("matches", "starts_with", "ends_with", "wildcard", "~")

# Rules in match order. Static assets first so they win over the HTML rule.
# Expressions are deliberately substring-based (see ALLOWED_OPERATORS).
#
# TTL FLOORS (Cloudflare plan limits — these are why the numbers look odd):
#   Minimum Edge Cache TTL:  Free 2h | Pro 1h | Business 1s | Enterprise 1s
#   Minimum Browser TTL:     1s on every plan
# So on Free, an edge TTL below 7200s is rejected. That makes HTML edge caching
# a bad default: the floor would hold a stale page for up to two hours. The HTML
# rule is therefore opt-in (--cache-html) and is not part of the default set.
FREE_MIN_EDGE_TTL = 7200

STATIC_RULE = {
    "description": f"{TAG} static assets — long cache",
    "expression": ('(http.request.uri.path eq "/app.js" '
                   'or http.request.uri.path eq "/favicon.svg" '
                   'or (http.request.uri.path contains "/og-" '
                   'and http.request.uri.path contains ".png"))'),
    "action": "set_cache_settings",
    "action_parameters": {
        "cache": True,
        "edge_ttl": {"mode": "override_origin", "default": 31536000},   # 1 year
        "browser_ttl": {"mode": "override_origin", "default": 31536000},
    },
}

# Opt-in only. See TTL FLOORS above: 2h of staleness is the Free-plan floor.
HTML_RULE = {
    "description": f"{TAG} HTML — short cache",
    "expression": ('(http.request.uri.path eq "/" '
                   'or http.request.uri.path contains ".html")'),
    "action": "set_cache_settings",
    "action_parameters": {
        "cache": True,
        # Free floor is 2h; on paid plans this can drop to 300s.
        "edge_ttl": {"mode": "override_origin", "default": FREE_MIN_EDGE_TTL},
        # respect_origin: the origin already sends max-age=600, and a lower
        # override would be ignored anyway (Cloudflare only raises browser TTL).
        "browser_ttl": {"mode": "respect_origin"},
    },
}

CRAWLER_RULE = {
    "description": f"{TAG} crawler files — medium cache",
    "expression": ('(http.request.uri.path in '
                   '{"/sitemap.xml" "/robots.txt" "/llms.txt" "/humans.txt"})'),
    "action": "set_cache_settings",
    "action_parameters": {
        "cache": True,
        "edge_ttl": {"mode": "override_origin", "default": FREE_MIN_EDGE_TTL},  # Free floor
        "browser_ttl": {"mode": "override_origin", "default": FREE_MIN_EDGE_TTL},
    },
}


def build_rules(cache_html=False):
    """The rules to deploy. HTML is opt-in (see TTL FLOORS above)."""
    rules = [STATIC_RULE]
    if cache_html:
        rules.append(HTML_RULE)
    rules.append(CRAWLER_RULE)
    return rules


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


def self_test():
    """Verify the rule expressions and merge logic without touching Cloudflare.

    Each expression is translated to Python and evaluated against real paths from
    the live site, so the assertions run against the exact strings we send to the
    API — not a hand-written paraphrase of them. Both rule sets are checked: the
    default (no HTML caching) and the --cache-html variant.
    """
    import re

    def to_python(expr):
        s = re.sub(r'http\.request\.uri\.path\s+eq\s+"([^"]*)"', r'(p == "\1")', expr)
        s = re.sub(r'http\.request\.uri\.path\s+contains\s+"([^"]*)"', r'("\1" in p)', s)
        s = re.sub(r'http\.request\.uri\.path\s+in\s+\{([^}]*)\}',
                   # the captured tokens already carry their quotes: join with commas
                   lambda m: "p in {" + ", ".join(m.group(1).split()) + "}", s)
        return s

    failures = []

    def check_routing(rules, expected, label):
        def hits(path):
            return [i for i, r in enumerate(rules)
                    if eval(to_python(r["expression"]), {"p": path})]
        for path, want in expected.items():
            got = hits(path)
            if got != want:
                failures.append(f"[{label}] {path}: matched {got}, expected {want}")
            if len(got) > 1:
                failures.append(f"[{label}] {path}: ambiguous, matches {got}")

    common_static = {"/app.js": [0], "/favicon.svg": [0],
                     "/og-pricing.png": [0], "/og-image.zh.png": [0]}
    crawler = {"/sitemap.xml": None, "/robots.txt": None, "/llms.txt": None, "/humans.txt": None}

    # Default set: [static, crawler] — HTML is deliberately NOT edge-cached.
    default_rules = build_rules(cache_html=False)
    exp = dict(common_static)
    exp.update({"/": [], "/index.html": [], "/nonexistent": []})
    exp.update({p: [1] for p in crawler})
    check_routing(default_rules, exp, "default")

    # --cache-html set: [static, html, crawler].
    html_rules = build_rules(cache_html=True)
    exp = dict(common_static)
    exp.update({"/": [1], "/index.html": [1], "/pricing.zh.html": [1], "/404.html": [1],
                "/nonexistent": []})
    exp.update({p: [2] for p in crawler})
    check_routing(html_rules, exp, "cache-html")

    # Operators must work on every plan. `matches` needs Business/Enterprise and
    # `starts_with`/`ends_with` are unsupported in custom rules — either one gets
    # the whole ruleset rejected with HTTP 400.
    for r in html_rules:
        for bad in FORBIDDEN_OPERATORS:
            if bad in r["expression"]:
                failures.append(f"{r['description']}: uses forbidden operator {bad!r}")

    # TTL enums must be real Cloudflare variants, and edge TTL must clear the
    # Free-plan floor (2h) or the API rejects the ruleset.
    for r in html_rules:
        ap = r["action_parameters"]
        for key in ("edge_ttl", "browser_ttl"):
            mode = ap[key]["mode"]
            if mode not in TTL_MODES:
                failures.append(f"{r['description']}: {key}.mode={mode!r} not in {sorted(TTL_MODES)}")
            if mode == "override_origin":
                val = ap[key].get("default")
                if not isinstance(val, int):
                    failures.append(f"{r['description']}: {key} override_origin needs an int default")
                elif key == "edge_ttl" and val < FREE_MIN_EDGE_TTL:
                    failures.append(
                        f"{r['description']}: edge_ttl={val}s below Free-plan floor {FREE_MIN_EDGE_TTL}s")
        if r["action"] != "set_cache_settings":
            failures.append(f"{r['description']}: action must be set_cache_settings")

    # Merge logic: our rules replace ours, never the user's.
    user_rule = {"description": "someone else's rule", "expression": "true",
                 "action": "set_cache_settings", "action_parameters": {}}
    existing = [user_rule] + [dict(r) for r in default_rules] + [user_rule]
    kept = [r for r in existing if not str(r.get("description", "")).startswith(TAG)]
    ours = [r for r in existing if str(r.get("description", "")).startswith(TAG)]
    if len(kept) != 2 or len(ours) != 2:
        failures.append(f"merge: kept {len(kept)} (want 2), ours {len(ours)} (want 2)")
    if len(({"rules": default_rules + kept})["rules"]) != 4:
        failures.append("merge: expected 4 rules after merge")

    if failures:
        print("SELF-TEST FAILED:")
        for f in failures:
            print("  -", f)
        return 1

    print(f"self-test OK: 两种规则集路由正确、互不重叠；算符全套餐可用；edge_ttl ≥ {FREE_MIN_EDGE_TTL}s（Free 下限）")
    print(f"\n默认规则集（{len(default_rules)} 条，不缓存 HTML）：")
    for p in ("/", "/index.html", "/app.js", "/og-pricing.png", "/sitemap.xml"):
        got = [i for i, r in enumerate(default_rules)
               if eval(to_python(r["expression"]), {"p": p})]
        desc = default_rules[got[0]]["description"] if got else "（不缓存，走源站）"
        print(f"  {p:20} -> {desc}")
    print(f"\n--cache-html 规则集（{len(html_rules)} 条）：")
    for p in ("/", "/index.html"):
        got = [i for i, r in enumerate(html_rules) if eval(to_python(r["expression"]), {"p": p})]
        print(f"  {p:20} -> {html_rules[got[0]]['description']}")
    return 0


def main():
    ap = argparse.ArgumentParser(description="Apply aipcmaster.com Cloudflare cache rules")
    ap.add_argument("--zone", default="aipcmaster.com", help="zone name (default: aipcmaster.com)")
    ap.add_argument("--zone-id", help="zone id, skips the lookup")
    ap.add_argument("--dry-run", action="store_true", help="print the payload, change nothing")
    ap.add_argument("--verify-only", action="store_true", help="only check live headers")
    ap.add_argument("--self-test", action="store_true", help="validate rules locally, no network")
    ap.add_argument("--print-payload", action="store_true", help="print the exact JSON to be sent, no network")
    ap.add_argument("--cache-html", action="store_true",
                    help="also edge-cache HTML (Free plan floor is 2h, so pages can be stale that long)")
    ap.add_argument("--purge", action="store_true",
                    help="purge the whole edge cache (needs Zone → Cache Purge → Purge)")
    ap.add_argument("--purge-urls", metavar="URLS",
                    help="comma-separated URLs to purge instead of everything")
    args = ap.parse_args()

    rules = build_rules(cache_html=args.cache_html)

    if args.self_test:
        raise SystemExit(self_test())

    if args.print_payload:
        print(json.dumps({"rules": rules}, ensure_ascii=False, indent=2))
        return

    if args.verify_only:
        verify()
        return

    if args.purge or args.purge_urls:
        token = require_token()
        zid = find_zone(token, args.zone, args.zone_id)
        files = ([u.strip() for u in args.purge_urls.split(",") if u.strip()]
                 if args.purge_urls else None)
        target = f"{len(files)} 个 URL" if files else "全部"
        print(f"zone: {args.zone}  ({zid})")
        print(f"清理边缘缓存：{target}")
        result = purge(token, zid, files)
        print(f"✅ 已提交清理（{result.get('id', 'ok')}）")
        if files:
            for f in files:
                print(f"   - {f}")
        print("\n边缘副本已删除；下次请求会 MISS 回源，随后重新 HIT。")
        print("源站文件未受影响。")
        print()
        verify()
        return

    token = require_token()

    zid = find_zone(token, args.zone, args.zone_id)
    print(f"zone: {args.zone}  ({zid})")

    existing = get_existing(token, zid)
    kept = [r for r in existing if not str(r.get("description", "")).startswith(TAG)]
    ours = [r for r in existing if str(r.get("description", "")).startswith(TAG)]
    print(f"现有 cache 规则 {len(existing)} 条：保留 {len(kept)} 条，替换本工具的 {len(ours)} 条")

    payload = {"rules": rules + kept}  # ours first: match order is top-down

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


def require_token():
    """Token comes from the environment only — never argv, never written to disk."""
    token = os.environ.get("CF_API_TOKEN")
    if not token:
        raise SystemExit(
            "未设置 CF_API_TOKEN。\n"
            "  CF_API_TOKEN=xxxxx python3 deploy/cloudflare/apply.py --zone aipcmaster.com\n"
            "（token 只从环境变量读取，避免留在 shell 历史里）"
        )
    return token


def purge(token, zid, files=None):
    """Drop cached copies at the edge so the next request refetches from origin.

    Does not touch origin files — only Cloudflare's edge copies. Everything is
    the right default for a site this size; per-URL exists to be surgical.
    Needs the token scope: Zone → Cache Purge → Purge.
    """
    body = {"files": files} if files else {"purge_everything": True}
    res = request("POST", f"/zones/{zid}/purge_cache", token, body)
    return res.get("result") or {}


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
