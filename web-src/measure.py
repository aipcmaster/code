#!/usr/bin/env python3
"""Measure Core Web Vitals + resource cost for the local site build.

Serves web/ on localhost, loads it with the system Chrome via Playwright,
and reports LCP, CLS, FCP, TTFB, DOM size and per-resource transfer sizes.

Usage:  python3 measure.py [--json]
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
PORT = 8731

METRICS_JS = """
() => new Promise((resolve) => {
  const out = { lcp: 0, cls: 0, fcp: 0, shifts: [] };
  try {
    new PerformanceObserver((l) => {
      for (const e of l.getEntries()) out.lcp = Math.max(out.lcp, e.startTime);
    }).observe({ type: 'largest-contentful-paint', buffered: true });
    new PerformanceObserver((l) => {
      for (const e of l.getEntries()) {
        if (!e.hadRecentInput) { out.cls += e.value; out.shifts.push(e.value); }
      }
    }).observe({ type: 'layout-shift', buffered: true });
    new PerformanceObserver((l) => {
      for (const e of l.getEntries()) if (e.name === 'first-contentful-paint') out.fcp = e.startTime;
    }).observe({ type: 'paint', buffered: true });
  } catch (e) { out.err = String(e); }
  // give observers a beat, then collect nav + resources
  setTimeout(() => {
    const nav = performance.getEntriesByType('navigation')[0] || {};
    out.ttfb = nav.responseStart || 0;
    out.domContentLoaded = nav.domContentLoadedEventEnd || 0;
    out.load = nav.loadEventEnd || 0;
    out.resources = performance.getEntriesByType('resource').map((r) => ({
      name: r.name.split('/').pop(),
      transfer: r.transferSize || 0,
      encoded: r.encodedBodySize || 0,
      duration: Math.round(r.duration),
    }));
    out.domNodes = document.getElementsByTagName('*').length;
    resolve(out);
  }, 600);
})
"""


def serve():
    handler = functools.partial(http.server.SimpleHTTPRequestHandler, directory=str(WEB))
    httpd = socketserver.ThreadingTCPServer(("127.0.0.1", PORT), handler)
    httpd.daemon_threads = True
    threading.Thread(target=httpd.serve_forever, daemon=True).start()
    return httpd


def measure(page_path="/index.html"):
    from playwright.sync_api import sync_playwright

    with sync_playwright() as p:
        browser = p.chromium.launch(
            executable_path="/usr/bin/google-chrome",
            args=["--no-sandbox", "--disable-gpu"],
        )
        page = browser.new_page(viewport={"width": 1280, "height": 900})
        # Warm-up navigation: isolates Chrome process/first-run cost from the page's own cost.
        page.goto(f"http://127.0.0.1:{PORT}{page_path}", wait_until="load")
        page.wait_for_timeout(300)
        page.goto("about:blank")
        page.goto(f"http://127.0.0.1:{PORT}{page_path}", wait_until="load")
        data = page.evaluate(METRICS_JS)
        browser.close()
    return data


def main():
    httpd = serve()
    try:
        data = measure()
    finally:
        httpd.shutdown()

    as_json = "--json" in sys.argv
    if as_json:
        print(json.dumps(data, ensure_ascii=False, indent=2))
        return

    print("Core Web Vitals (lab, system Chrome, localhost)")
    print(f"  FCP               {data['fcp']:.0f} ms")
    print(f"  LCP               {data['lcp']:.0f} ms")
    print(f"  CLS               {data['cls']:.4f}")
    print(f"  TTFB              {data['ttfb']:.0f} ms")
    print(f"  DOMContentLoaded  {data['domContentLoaded']:.0f} ms")
    print(f"  Load              {data['load']:.0f} ms")
    print(f"  DOM nodes         {data['domNodes']}")
    print("\nResources")
    total = 0
    for r in sorted(data["resources"], key=lambda x: -x["transfer"]):
        total += r["transfer"]
        print(f"  {r['transfer']:>8,} B  {r['name']:<20} {r['duration']} ms")
    print(f"  {total:>8,} B  TOTAL transfer")


if __name__ == "__main__":
    main()
