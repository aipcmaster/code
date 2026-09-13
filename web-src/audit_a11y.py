#!/usr/bin/env python3
"""Accessibility audit: runs axe-core (WCAG 2.1 A/AA) against every generated page.

Serves web/, injects the locally cached axe-core bundle into each page with
Playwright, and reports violations grouped by rule with impact and affected nodes.

Usage:
  python3 audit_a11y.py                 # all pages
  python3 audit_a11y.py index.html      # one page
  python3 audit_a11y.py --json          # machine-readable
"""
import functools
import http.server
import json
import pathlib
import socketserver
import sys
import threading

ROOT = pathlib.Path(__file__).parent.parent
WEB = ROOT / "web"
PORT = 8791
AXE = pathlib.Path(
    "/home/jackliao/.bun/install/cache/axe-core@4.11.4@@registry.npmmirror.com@@@1/axe.min.js"
)

AXE_OPTIONS = {
    "runOnly": {"type": "tag", "values": ["wcag2a", "wcag2aa", "wcag21a", "wcag21aa", "best-practice"]},
    "resultTypes": ["violations"],
}


class Server(socketserver.ThreadingTCPServer):
    allow_reuse_address = True
    daemon_threads = True


def serve():
    handler = functools.partial(http.server.SimpleHTTPRequestHandler, directory=str(WEB))
    handler.log_message = lambda *a, **k: None
    httpd = Server(("127.0.0.1", PORT), handler)
    threading.Thread(target=httpd.serve_forever, daemon=True).start()
    return httpd


def audit(pages):
    from playwright.sync_api import sync_playwright

    results = {}
    with sync_playwright() as p:
        browser = p.chromium.launch(executable_path="/usr/bin/google-chrome",
                                    args=["--no-sandbox", "--disable-gpu"])
        page = browser.new_page(viewport={"width": 1280, "height": 900})
        for name in pages:
            page.goto(f"http://127.0.0.1:{PORT}/{name}", wait_until="load")
            page.add_script_tag(path=str(AXE))  # per-navigation: goto resets the document
            res = page.evaluate(
                "async (opts) => { const r = await axe.run(document, opts); "
                "return r.violations.map(v => ({ id: v.id, impact: v.impact, help: v.help, "
                "nodes: v.nodes.map(n => ({ target: n.target.join(' '), summary: n.failureSummary })) })); }",
                AXE_OPTIONS,
            )
            results[name] = res
        browser.close()
    return results


def main():
    if AXE.exists() is False:
        print(f"axe-core not found at {AXE}")
        return 1

    if "--json" in sys.argv:
        pages = sorted(f.name for f in WEB.glob("*.html"))
    elif len(sys.argv) > 1 and sys.argv[1].endswith(".html"):
        pages = [sys.argv[1]]
    else:
        pages = sorted(f.name for f in WEB.glob("*.html"))

    httpd = serve()
    try:
        results = audit(pages)
    finally:
        httpd.shutdown()

    if "--json" in sys.argv:
        print(json.dumps(results, ensure_ascii=False, indent=2))
        return 0

    # Group by rule across pages.
    by_rule = {}
    total = 0
    for name, violations in results.items():
        for v in violations:
            total += len(v["nodes"])
            by_rule.setdefault(v["id"], {"impact": v["impact"], "help": v["help"], "pages": {}})
            by_rule[v["id"]]["pages"].setdefault(name, []).extend(n["target"] for n in v["nodes"])

    print(f"axe-core WCAG 2.1 A/AA + best-practice — {len(pages)} pages")
    if not by_rule:
        print("\n  No violations. Clean.")
        return 0

    print(f"\n  {len(by_rule)} rule(s), {total} node(s) affected\n")
    order = {"critical": 0, "serious": 1, "moderate": 2, "minor": 3, None: 4}
    for rid, info in sorted(by_rule.items(), key=lambda kv: order.get(kv[1]["impact"], 4)):
        pages_hit = info["pages"]
        n = sum(len(v) for v in pages_hit.values())
        print(f"  [{info['impact']}] {rid} — {info['help']}")
        print(f"      {n} node(s) across {len(pages_hit)} page(s): {', '.join(sorted(pages_hit))}")
        for pg, targets in list(pages_hit.items())[:1]:
            for t in targets[:3]:
                print(f"        e.g. {t}")
        print()
    return 1


if __name__ == "__main__":
    sys.exit(main())
