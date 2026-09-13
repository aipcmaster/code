#!/usr/bin/env python3
"""Before/after check: does inlining the CSS actually help under real latency?

Builds two variants of the generated homepage — CSS inlined (the current build) and
CSS as an external render-blocking stylesheet (the old shape) — serves each on
localhost, and measures FCP/LCP/CLS under slow-4G throttling via CDP.

This is the evidence behind the "inline the CSS" decision. Run after generate.py.

Usage:  python3 compare_cwv.py
"""
import functools
import http.server
import pathlib
import re
import shutil
import socketserver
import tempfile
import threading

from playwright.sync_api import sync_playwright

ROOT = pathlib.Path(__file__).parent.parent
SRC = ROOT / "web"

# Slow 4G: 150 ms RTT, ~1.6 Mbps down.
THROTTLE = {"latency": 150, "download": 200_000, "upload": 100_000}

METRICS_JS = """
() => new Promise((resolve) => {
  const out = { lcp: 0, fcp: 0, cls: 0 };
  new PerformanceObserver((l) => { for (const e of l.getEntries()) out.lcp = Math.max(out.lcp, e.startTime); })
    .observe({ type: 'largest-contentful-paint', buffered: true });
  new PerformanceObserver((l) => { for (const e of l.getEntries()) if (e.name === 'first-contentful-paint') out.fcp = e.startTime; })
    .observe({ type: 'paint', buffered: true });
  new PerformanceObserver((l) => { for (const e of l.getEntries()) if (!e.hadRecentInput) out.cls += e.value; })
    .observe({ type: 'layout-shift', buffered: true });
  setTimeout(() => resolve(out), 500);
})
"""


def build_variants(base):
    inline = base / "inline"
    external = base / "external"
    shutil.copytree(SRC, inline)
    shutil.copytree(SRC, external)

    html = (external / "index.html").read_text(encoding="utf-8")
    css = re.search(r"<style>(.*?)</style>", html, re.S).group(1)
    (external / "styles.css").write_text(css, encoding="utf-8")
    (external / "index.html").write_text(
        html.replace(f"<style>{css}</style>", '<link rel="stylesheet" href="styles.css">'),
        encoding="utf-8",
    )
    return inline, external


def serve(directory, port):
    handler = functools.partial(http.server.SimpleHTTPRequestHandler, directory=str(directory))
    handler.log_message = lambda *a, **k: None
    httpd = socketserver.ThreadingTCPServer(("127.0.0.1", port), handler)
    httpd.daemon_threads = True
    threading.Thread(target=httpd.serve_forever, daemon=True).start()
    return httpd


def run(browser, port):
    ctx = browser.new_context(viewport={"width": 1280, "height": 900})
    page = ctx.new_page()
    cdp = ctx.new_cdp_session(page)
    cdp.send("Network.enable")
    cdp.send("Network.emulateNetworkConditions", {
        "offline": False,
        "latency": THROTTLE["latency"],
        "downloadThroughput": THROTTLE["download"],
        "uploadThroughput": THROTTLE["upload"],
    })
    page.goto(f"http://127.0.0.1:{port}/index.html", wait_until="load")
    data = page.evaluate(METRICS_JS)
    ctx.close()
    return data


def main():
    with tempfile.TemporaryDirectory() as tmp:
        inline, external = build_variants(pathlib.Path(tmp))
        s1 = serve(external, 8741)
        s2 = serve(inline, 8742)
        try:
            with sync_playwright() as p:
                browser = p.chromium.launch(executable_path="/usr/bin/google-chrome",
                                            args=["--no-sandbox", "--disable-gpu"])
                ext = run(browser, 8741)
                inl = run(browser, 8742)
                browser.close()
        finally:
            s1.shutdown()
            s2.shutdown()

    print("Slow-4G throttled (150 ms RTT, ~1.6 Mbps)")
    print(f"  {'':22}{'FCP':>9}{'LCP':>9}{'CLS':>9}")
    for name, d in (("external CSS (before)", ext), ("inlined CSS (after)", inl)):
        print(f"  {name:20}{d['fcp']:>7.0f}ms{d['lcp']:>7.0f}ms{d['cls']:>9.4f}")
    print(f"\n  FCP saved: {ext['fcp'] - inl['fcp']:.0f} ms")
    print(f"  LCP saved: {ext['lcp'] - inl['lcp']:.0f} ms")


if __name__ == "__main__":
    main()
