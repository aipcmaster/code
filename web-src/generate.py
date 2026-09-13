#!/usr/bin/env python3
"""Generate the AIPCMaster site's static pages.

Why a generator: twenty-two pages, each in two languages, all sharing one nav, one footer and
one <head>. Hand-maintaining that means twenty-two places to forget a link. This writes plain
static HTML once, so what deploys is still just files — no runtime include, no JS needed to see
the navigation.

Run:  python3 generate.py     (writes into ../web/)
"""
import functools
import json
import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).parent))
from content import NAV, CTA, OTHER, PAGES  # noqa: E402

ROOT = pathlib.Path(__file__).parent.parent
OUT = ROOT / "web"
SITE = "https://aipcmaster.com"
YEAR = "2026"


def og_image(slug, lang):
    """Social preview image for a page. 404 has none, so it falls back to the default."""
    if slug == "index" or slug == "404":
        return "og-image.png" if lang == "en" else "og-image.zh.png"
    return f"og-{slug}.png" if lang == "en" else f"og-{slug}.zh.png"


# ── CSS: kept as one source file and inlined into every page ───────────────────
# Inlining removes the one render-blocking request this site had, so first paint
# no longer waits on a round trip. The sheet is small (and gzips with the HTML),
# and every page is a plausible landing page from search, so cross-page caching
# is not worth the blocking cost.

CSS_SOURCE = pathlib.Path(__file__).parent / "styles.css"


def minify_css(css):
    """Conservative, string-aware CSS minifier.

    Drops comments (keeping /*! ... */), collapses whitespace, and removes space
    around structural punctuation. It never touches quoted strings or url() bodies.
    """
    out = []
    i, n = 0, len(css)
    drop_if_prev = set("{};,>+~:")
    drop_if_next = set("{};,>+~")
    while i < n:
        c = css[i]
        if c == "/" and i + 1 < n and css[i + 1] == "*":
            keep = css.startswith("/*!", i)
            end = css.find("*/", i + 2)
            if keep:
                out.append(css[i:end + 2] if end != -1 else css[i:])
            i = end + 2 if end != -1 else n
            continue
        if c in "\"'":
            j = i + 1
            while j < n:
                if css[j] == "\\":
                    j += 2
                    continue
                if css[j] == c:
                    j += 1
                    break
                j += 1
            out.append(css[i:j])
            i = j
            continue
        if c in " \t\n\r\f":
            j = i
            while j < n and css[j] in " \t\n\r\f":
                j += 1
            prev = out[-1][-1] if out else ""
            nxt = css[j] if j < n else ""
            if prev in drop_if_prev or nxt in drop_if_next:
                i = j
                continue
            out.append(" ")
            i = j
            continue
        out.append(c)
        i += 1
    return "".join(out).replace(";} ", "}").replace(";}", "}").strip()


@functools.lru_cache(maxsize=1)
def load_css():
    return minify_css(CSS_SOURCE.read_text(encoding="utf-8"))

LOGO = """<svg viewBox="0 0 24 24" fill="none" aria-hidden="true">
        <rect x="3" y="4" width="18" height="13" rx="2.5" stroke="#2FE0A8" stroke-width="1.6"/>
        <path d="M8 20h8M12 17v3" stroke="#2FE0A8" stroke-width="1.6" stroke-linecap="round"/>
        <path d="M7.5 10.5l2 2 3-4 2 2.5 2-2" stroke="#E9EDF3" stroke-width="1.5"
              stroke-linecap="round" stroke-linejoin="round"/>
      </svg>"""


def href(lang, slug):
    """Page filename for a slug in a language. index has no slug suffix in either language."""
    if slug == "index":
        return "index.html" if lang == "en" else "index.zh.html"
    return f"{slug}.html" if lang == "en" else f"{slug}.zh.html"


def page_url(lang, slug):
    """Canonical absolute URL. The English homepage lives at the site root."""
    if slug == "index" and lang == "en":
        return f"{SITE}/"
    return f"{SITE}/{href(lang, slug)}"


def _ld(data):
    """Serialize JSON-LD compactly, keeping non-ASCII readable."""
    return json.dumps(data, ensure_ascii=False, separators=(",", ":"))


# ── FAQ: one source, used for both the visible section and FAQPage schema ──────
# Answers are deliberately short, factual and self-contained so that answer
# engines (ChatGPT, Claude, Perplexity, Google AI Overviews) can quote them.

FAQ = {
    "en": [
        ("What is AIPCMaster?",
         "AIPCMaster (AI电脑大师) is an AI-powered PC management tool for Windows 10 and 11. "
         "It reads CPU, memory, disk, temperature and driver data on the device itself, explains "
         "in plain language what is actually wrong, and applies only the optimizations you approve."),
        ("Does AIPCMaster send my data to the cloud?",
         "No. Collection and inference run locally on your PC. The cloud is used only for your "
         "account, device list and subscription. Telemetry stays on the device unless you export it."),
        ("Does it change my system without asking?",
         "No. It is read-only by default. Every system change requires your confirmation, a restore "
         "point is created before the change, and each action can be rolled back. Every action is logged."),
        ("How much does it cost?",
         "Every new account gets the full product free for 14 days, no card required. After that the "
         "free tier keeps basic diagnostics. Personal is ¥199/year, Family ¥349/year for up to 5 devices, "
         "and Business is ¥99 per device per year."),
        ("Which platforms are supported?",
         "Windows 10 and 11 are supported at launch, with V1.0 shipping in Q4 2026. macOS is planned "
         "for V1.5 in Q2 2027."),
        ("How is it different from a cleaner or a task manager?",
         "Those show a snapshot of numbers. AIPCMaster collects CPU, memory, disk, temperature, process, "
         "startup and driver data every 5 seconds (every 500 ms when something looks wrong), finds the "
         "root cause, predicts problems from the trend, and only then suggests a change."),
    ],
    "zh": [
        ("AIPCMaster（AI电脑大师）是什么？",
         "AI电脑大师是面向 Windows 10 / 11 的 AI 电脑管理工具。它在设备本地读取 CPU、内存、磁盘、"
         "温度与驱动数据，用大白话讲清到底哪里出了问题，并且只执行你确认过的优化。"),
        ("会把我的数据传到云端吗？",
         "不会。采集与推理都在本地完成，云端只负责账号、设备列表与订阅。除非你主动导出，"
         "遥测数据始终留在设备本地。"),
        ("它会不经允许就修改我的系统吗？",
         "不会。默认只读模式。每一次系统修改都需要你确认，改动前自动创建系统还原点，"
         "并且全程可回滚，每一步操作都有审计日志。"),
        ("价格是多少？",
         "每个新账号都有 14 天全功能免费试用，无需绑定银行卡。到期后免费版保留基础诊断。"
         "个人版 ¥199/年，家庭版 ¥349/年（最多 5 台设备），企业版 ¥99/设备/年。"),
        ("支持哪些平台？",
         "首发支持 Windows 10 与 11，V1.0 将于 2026 Q4 发布。macOS 计划在 2027 Q2 的 V1.5 版本支持。"),
        ("和普通的清理工具、任务管理器有什么区别？",
         "那些工具只给你一个当下的数字快照。AI电脑大师正常每 5 秒、异常时每 500 毫秒采集 CPU、内存、"
         "磁盘、温度、进程、启动项与驱动数据，先定位根因、再根据趋势预判问题，最后才给出改动建议。"),
    ],
}


def faq_html(lang):
    """Visible FAQ section (mirrors FAQPage schema)."""
    head = ("FREQUENTLY ASKED", "Questions people ask before installing") if lang == "en" \
        else ("常见问题", "安装之前，大家会问的问题")
    items = "\n".join(
        f"""      <details class="faq-item">
        <summary>{q}</summary>
        <p>{a}</p>
      </details>""" for q, a in FAQ[lang]
    )
    return f"""
<section id="faq">
  <div class="wrap">
    <div class="sec-head">
      <span class="sec-tag">{head[0]}</span>
      <h2>{head[1]}</h2>
    </div>
    <div class="faq">
{items}
    </div>
  </div>
</section>
"""


# ── Structured data (schema.org JSON-LD) ──────────────────────────────────────

ORG_DESC = {
    "en": "AIPCMaster builds on-device AI software that diagnoses and optimizes Windows PCs, "
          "explaining problems in plain language and changing only what the user approves.",
    "zh": "AIPCMaster（AI电脑大师）研发端侧 AI 电脑管理软件，诊断并优化 Windows 电脑，"
          "用大白话解释问题，且只执行用户确认过的改动。",
}

SOFTWARE_DESC = {
    "en": "AI-powered PC diagnostics and optimization for Windows. Reads CPU, memory, disk, "
          "temperature and drivers on the device, finds the root cause, and applies only approved changes.",
    "zh": "面向 Windows 的端侧 AI 电脑诊断与优化工具。在本地读取 CPU、内存、磁盘、温度与驱动状态，"
          "定位根因，并且只执行经用户确认的改动。",
}

OFFERS = [
    ("Trial", "0", "14-day full trial", "en"),
    ("Personal", "199", "per year", "en"),
    ("Family", "349", "per year, up to 5 devices", "en"),
    ("Business", "99", "per device per year", "en"),
]


def _offer_nodes(lang):
    names = {
        "Trial": ("试用版", "14 天全功能试用"),
        "Personal": ("个人版", "每年"),
        "Family": ("家庭版", "每年，最多 5 台设备"),
        "Business": ("企业版", "每台设备每年"),
    }
    nodes = []
    for en_name, price, desc_en, _ in OFFERS:
        zh_name, desc_zh = names[en_name]
        nodes.append({
            "@type": "Offer",
            "name": zh_name if lang == "zh" else en_name,
            "description": desc_zh if lang == "zh" else desc_en,
            "price": price,
            "priceCurrency": "CNY",
            "availability": "https://schema.org/PreOrder",
            "url": page_url(lang, "pricing"),
        })
    return nodes


def _breadcrumb(lang, slug, title):
    home = "Home" if lang == "en" else "首页"
    items = [{"@type": "ListItem", "position": 1, "name": home, "item": page_url(lang, "index")}]
    if slug != "index":
        items.append({"@type": "ListItem", "position": 2, "name": title, "item": page_url(lang, slug)})
    return {"@type": "BreadcrumbList", "itemListElement": items}


def _faq_schema(lang):
    return {
        "@type": "FAQPage",
        "mainEntity": [
            {"@type": "Question", "name": q,
             "acceptedAnswer": {"@type": "Answer", "text": a}}
            for q, a in FAQ[lang]
        ],
    }


def structured_data(lang, slug, title):
    """One @graph per page: Organization + WebSite, plus page-specific nodes."""
    org_id = f"{SITE}/#organization"
    site_id = f"{SITE}/#website"
    graph = [
        {
            "@type": "Organization",
            "@id": org_id,
            "name": "AIPCMaster",
            "alternateName": "AI电脑大师",
            "url": f"{SITE}/",
            "logo": {"@type": "ImageObject", "url": f"{SITE}/{og_image('index', lang)}", "width": 1200, "height": 630},
            "image": f"{SITE}/{og_image('index', lang)}",
            "description": ORG_DESC[lang],
            "foundingDate": YEAR,
            "sameAs": [],
            "contactPoint": [
                {"@type": "ContactPoint", "contactType": "customer support",
                 "email": "support@aipcmaster.com", "availableLanguage": ["en", "zh"]},
                {"@type": "ContactPoint", "contactType": "sales",
                 "email": "business@aipcmaster.com", "availableLanguage": ["en", "zh"]},
                {"@type": "ContactPoint", "contactType": "security",
                 "email": "security@aipcmaster.com", "availableLanguage": ["en", "zh"]},
            ],
        },
        {
            "@type": "WebSite",
            "@id": site_id,
            "url": f"{SITE}/",
            "name": "AIPCMaster",
            "alternateName": "AI电脑大师",
            "publisher": {"@id": org_id},
            "inLanguage": ["en", "zh-Hans"],
        },
        _breadcrumb(lang, slug, title),
    ]

    if slug == "index":
        graph.append({
            "@type": "SoftwareApplication",
            "@id": f"{SITE}/#software",
            "name": "AIPCMaster",
            "alternateName": "AI电脑大师",
            "applicationCategory": "UtilitiesApplication",
            "applicationSubCategory": "System Optimization",
            "operatingSystem": "Windows 10, Windows 11",
            "softwareVersion": "1.0",
            "releaseNotes": "V1.0 ships Q4 2026 with core diagnostics and optimization for Windows.",
            "description": SOFTWARE_DESC[lang],
            "inLanguage": ["en", "zh-Hans"],
            "publisher": {"@id": org_id},
            "featureList": [
                "One-click diagnostics with a 0-100 health score",
                "Root-cause analysis in plain language",
                "Automatic optimization with restore point and one-click rollback",
                "Predictive maintenance from trend data",
                "On-device inference with a full audit log",
            ] if lang == "en" else [
                "一键诊断，输出 0-100 健康分",
                "用大白话给出根因分析",
                "自动优化，改动前创建还原点，支持一键回滚",
                "基于趋势数据的预测性维护",
                "端侧推理，完整审计日志",
            ],
            "offers": _offer_nodes(lang),
        })
        graph.append(_faq_schema(lang))

    if slug == "pricing":
        graph.append({
            "@type": "Product",
            "name": "AIPCMaster",
            "alternateName": "AI电脑大师",
            "description": SOFTWARE_DESC[lang],
            "brand": {"@type": "Brand", "name": "AIPCMaster"},
            "category": "SoftwareApplication",
            "image": f"{SITE}/{og_image(slug, lang)}",
            "offers": _offer_nodes(lang),
        })

    return {"@context": "https://schema.org", "@graph": graph}



def nav(lang):
    home = href(lang, "index")
    cta_label, cta_slug = CTA[lang]
    other_label, other_href = OTHER[lang]
    links = "\n".join(
        f'      <a href="{h}">{t}</a>' for t, h in NAV[lang]
    )
    brand = "AIPCMaster" if lang == "en" else "AI电脑大师"
    brand_alt = "AI电脑大师" if lang == "en" else "AIPCMaster"
    return f"""<nav class="nav">
  <div class="wrap nav-in">
    <a class="brand" href="{home}">
      {LOGO}
      <b>{brand}</b><span>{brand_alt}</span>
    </a>
    <div class="nav-links" id="nav-links">
{links}
    </div>
    <a class="lang" href="{other_href}" hreflang="{'zh-Hans' if lang == 'en' else 'en'}">{other_label}</a>
    <a class="btn btn-primary" href="{href(lang, cta_slug.split('.')[0])}">{cta_label}</a>
    <button class="nav-toggle" id="nav-toggle" aria-label="Menu" aria-expanded="false" aria-controls="nav-links">
      <span></span><span></span><span></span>
    </button>
  </div>
</nav>"""


def footer(lang):
    home = href(lang, "index")
    if lang == "en":
        blurb = "AI-powered PC management. On-device first, predictive, safe by design."
        cols = [
            ("Product", [("Download", "download.html"), ("Pricing", "pricing.html"),
                         ("Docs", "docs.html"), ("Trial", "trial.html")]),
            ("Company", [("Business", "business.html"), ("Developers", "developers.html"),
                         ("Security", "security.html"), ("Support", "support.html")]),
            ("Contact", [("dev@aipcmaster.com", "mailto:dev@aipcmaster.com"),
                         ("support@aipcmaster.com", "mailto:support@aipcmaster.com"),
                         ("security@aipcmaster.com", "mailto:security@aipcmaster.com"),
                         ("business@aipcmaster.com", "mailto:business@aipcmaster.com")]),
        ]
        base_left = f"© {YEAR} AIPCMaster. All rights reserved."
        base_right = '<a href="privacy.html">Privacy</a> · <a href="terms.html">Terms</a> · <a href="index.zh.html">中文</a>'
        brand = "AIPCMaster"
    else:
        blurb = "端侧优先、智能预判、安全可控的 AI 电脑管理工具。"
        cols = [
            ("产品", [("下载", "download.zh.html"), ("价格", "pricing.zh.html"),
                      ("文档中心", "docs.zh.html"), ("试用申请", "trial.zh.html")]),
            ("公司", [("企业版", "business.zh.html"), ("开发者中心", "developers.zh.html"),
                      ("安全", "security.zh.html"), ("支持", "support.zh.html")]),
            ("联系", [("dev@aipcmaster.com", "mailto:dev@aipcmaster.com"),
                      ("support@aipcmaster.com", "mailto:support@aipcmaster.com"),
                      ("security@aipcmaster.com", "mailto:security@aipcmaster.com"),
                      ("business@aipcmaster.com", "mailto:business@aipcmaster.com")]),
        ]
        base_left = f"© {YEAR} AIPCMaster. All rights reserved."
        base_right = '<a href="privacy.zh.html">隐私政策</a> · <a href="terms.zh.html">用户协议</a> · <a href="index.html">English</a>'
        brand = "AI电脑大师"

    blocks = []
    for title, items in cols:
        lis = "\n".join(f'          <li><a href="{h}">{t}</a></li>' for t, h in items)
        blocks.append(f"""      <div>
        <h3>{title}</h3>
        <ul>
{lis}
        </ul>
      </div>""")
    cols_html = "\n".join(blocks)

    return f"""<footer>
  <div class="wrap">
    <div class="foot-grid">
      <div>
        <a class="brand" href="{home}" style="margin-bottom:10px">{LOGO}<b>{brand}</b></a>
        <p style="margin:0;max-width:26em">{blurb}</p>
      </div>
{cols_html}
    </div>
    <div class="foot-base">
      <span>{base_left}</span>
      <span>{base_right}</span>
    </div>
  </div>
</footer>"""


def shell(lang, slug, title, desc, body, noindex=False):
    html_lang = "en" if lang == "en" else "zh-Hans"
    url = page_url(lang, slug)
    en_url = page_url("en", slug)
    zh_url = page_url("zh", slug)
    suffix = "" if slug == "index" else " — AIPCMaster"
    if lang == "zh":
        suffix = "" if slug == "index" else " — AI电脑大师"
    full_title = f"{title}{suffix}" if slug != "index" else title
    og_img = f"{SITE}/{og_image(slug, lang)}"
    og_alt = ("AIPCMaster — on-device AI PC diagnostics and optimization"
              if lang == "en" else "AI电脑大师 — 端侧 AI 电脑诊断与优化")
    og_locale = "en_US" if lang == "en" else "zh_CN"
    og_locale_alt = "zh_CN" if lang == "en" else "en_US"
    robots = ("noindex, follow" if noindex
              else "index, follow, max-image-preview:large, max-snippet:-1, max-video-preview:-1")
    ld = _ld(structured_data(lang, slug, title))
    # Inner pages show no visible page title; give assistive tech and search engines
    # the document's h1 anyway, so every page has exactly one and heading order holds.
    page_h1 = "" if slug == "index" else f'<h1 class="sr-only">{title}</h1>\n'
    return f"""<!DOCTYPE html>
<html lang="{html_lang}">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{full_title}</title>
<meta name="description" content="{desc}">
<meta name="robots" content="{robots}">
<meta name="author" content="AIPCMaster">
<meta name="theme-color" content="#0A0C0F">
<meta name="color-scheme" content="dark">
<link rel="author" href="humans.txt">
<link rel="canonical" href="{url}">
<link rel="alternate" hreflang="en" href="{en_url}">
<link rel="alternate" hreflang="zh-Hans" href="{zh_url}">
<link rel="alternate" hreflang="x-default" href="{en_url}">
<meta property="og:type" content="website">
<meta property="og:site_name" content="AIPCMaster">
<meta property="og:locale" content="{og_locale}">
<meta property="og:locale:alternate" content="{og_locale_alt}">
<meta property="og:title" content="{full_title}">
<meta property="og:description" content="{desc}">
<meta property="og:url" content="{url}">
<meta property="og:image" content="{og_img}">
<meta property="og:image:type" content="image/png">
<meta property="og:image:width" content="1200">
<meta property="og:image:height" content="630">
<meta property="og:image:alt" content="{og_alt}">
<meta name="twitter:card" content="summary_large_image">
<meta name="twitter:title" content="{full_title}">
<meta name="twitter:description" content="{desc}">
<meta name="twitter:image" content="{og_img}">
<meta name="twitter:image:alt" content="{og_alt}">
<link rel="icon" href="favicon.svg" type="image/svg+xml">
<style>{load_css()}</style>
<script type="application/ld+json">{ld}</script>
</head>
<body>

{nav(lang)}

<main>
{page_h1}{body}
</main>

{footer(lang)}

<script src="app.js" defer></script>
</body>
</html>
"""


# ── the homepage body (kept here because its hero panel is bespoke) ────────────

INDEX_EN = """
<header class="hero">
  <div class="wrap hero-grid">
    <div>
      <span class="eyebrow"><i></i>Windows · V1.0 shipping Q4 2026</span>
      <h1>Your PC, looked after by AI — <em>before</em> it slows down.</h1>
      <p class="lede">
        AIPCMaster reads CPU, memory, disk, temperature and drivers on the device itself,
        explains what is actually wrong in plain language, and changes only what you approve.
      </p>
      <div class="hero-cta">
        <a class="btn btn-primary" href="download.html">Download for Windows</a>
        <a class="btn btn-ghost" href="pricing.html">See pricing</a>
      </div>
      <p class="hero-note">14-day full trial · no card required · on-device inference</p>
    </div>

    <div class="panel" aria-hidden="true">
      <div class="panel-top">
        <span class="dot"></span><span class="dot"></span><span class="dot"></span>
        <span class="panel-title">health · live</span>
      </div>
      <div class="score">
        <div class="ring"></div>
        <div class="score-meta"><b>System health 86</b><span>3 findings · 1 needs attention</span></div>
      </div>
      <div class="bars">
        <div class="bar-row"><span>CPU</span><span class="track"><span class="fill" style="width:34%"></span></span><span class="val">34%</span></div>
        <div class="bar-row"><span>Memory</span><span class="track"><span class="fill warn" style="width:81%"></span></span><span class="val">81%</span></div>
        <div class="bar-row"><span>Disk</span><span class="track"><span class="fill" style="width:52%"></span></span><span class="val">52%</span></div>
        <div class="bar-row"><span>Temp</span><span class="track"><span class="fill" style="width:46%"></span></span><span class="val">46°C</span></div>
      </div>
    </div>
  </div>
</header>

<section id="features">
  <div class="wrap">
    <div class="sec-head">
      <span class="sec-tag">CAPABILITIES</span>
      <h2>Four things it does, and does on your machine</h2>
      <p>Every check runs locally. The cloud is for account and subscription, not for watching your PC.</p>
    </div>
    <div class="grid-4">
      <div class="card">
        <div class="ico"><svg viewBox="0 0 24 24" fill="none"><circle cx="11" cy="11" r="6.5" stroke="currentColor" stroke-width="1.7"/><path d="M16 16l4 4" stroke="currentColor" stroke-width="1.7" stroke-linecap="round"/></svg></div>
        <h3>Smart diagnostics</h3>
        <p>One click, or ask in your own words — "why is my PC slow?" A health score, the bottleneck, the root cause, and a report you can export.</p>
      </div>
      <div class="card">
        <div class="ico"><svg viewBox="0 0 24 24" fill="none"><path d="M12 3v3M12 18v3M3 12h3M18 12h3M5.6 5.6l2.1 2.1M16.3 16.3l2.1 2.1M18.4 5.6l-2.1 2.1M7.7 16.3l-2.1 2.1" stroke="currentColor" stroke-width="1.6" stroke-linecap="round"/><circle cx="12" cy="12" r="3.2" stroke="currentColor" stroke-width="1.6"/></svg></div>
        <h3>Automatic optimization</h3>
        <p>Memory, startup items, disk cleanup, drivers, system settings. Read-only by default, a restore point before every change, and one-click rollback.</p>
      </div>
      <div class="card">
        <div class="ico"><svg viewBox="0 0 24 24" fill="none"><path d="M12 3l7 3v6c0 4.2-2.9 7.6-7 9-4.1-1.4-7-4.8-7-9V6l7-3z" stroke="currentColor" stroke-width="1.6" stroke-linejoin="round"/><path d="M12 8v4.5M12 16h.01" stroke="currentColor" stroke-width="1.7" stroke-linecap="round"/></svg></div>
        <h3>Predictive maintenance</h3>
        <p>Watches the trend, not just the moment: rising temperature, degrading disk, recurring stalls. Warns you before it becomes a repair.</p>
      </div>
      <div class="card">
        <div class="ico"><svg viewBox="0 0 24 24" fill="none"><rect x="4.5" y="10" width="15" height="10" rx="2.2" stroke="currentColor" stroke-width="1.6"/><path d="M8 10V7.5a4 4 0 018 0V10" stroke="currentColor" stroke-width="1.6"/></svg></div>
        <h3>Privacy by design</h3>
        <p>Telemetry stays on the device unless you export it. Full audit log of every action taken, and two-step verification on the account.</p>
      </div>
    </div>
  </div>
</section>

<section id="how">
  <div class="wrap">
    <div class="sec-head">
      <span class="sec-tag">ARCHITECTURE</span>
      <h2>Edge-cloud, in four layers</h2>
      <p>Collection and inference are local, so the machine keeps working with the network down.</p>
    </div>
    <div class="grid-4 flow">
      <div class="card"><h3>Collect</h3><p>CPU, memory, disk, network, temperature, processes, startup items, drivers. Every 5 seconds normally, every 500 ms when something looks wrong.</p></div>
      <div class="card"><h3>Infer</h3><p>A local LLM and light models on NPU or GPU. Anomaly detection, root-cause analysis, and optimisation suggestions — no round trip.</p></div>
      <div class="card"><h3>Decide</h3><p>Read-only by default. System changes need your confirmation, a restore point is created first, and everything can be rolled back.</p></div>
      <div class="card"><h3>Interact</h3><p>Natural-language chat, a live dashboard, diagnostic reports, the optimisation centre, and subscription management.</p></div>
    </div>
  </div>
</section>

<section id="pricing">
  <div class="wrap">
    <div class="sec-head">
      <span class="sec-tag">PRICING</span>
      <h2>Start free. Pay when it has earned it.</h2>
      <p>Every new account gets the full product for 14 days. After that, the free tier keeps basic diagnostics.</p>
    </div>
    <div class="price-grid">
      <div class="tier"><div class="name">Trial</div><div class="amount">Free <small>14 days</small></div><ul><li>Full diagnostics</li><li>Basic optimisation</li><li>No card required</li></ul></div>
      <div class="tier feature"><div class="name">Personal<span class="badge">POPULAR</span></div><div class="amount">¥199 <small>CNY / year</small></div><ul><li>Everything in Trial, permanently</li><li>Automatic optimisation</li><li>Predictive maintenance</li><li>Privacy controls</li></ul></div>
      <div class="tier"><div class="name">Family</div><div class="amount">¥349 <small>CNY / year</small></div><ul><li>Everything in Personal</li><li>Up to 5 devices</li><li>Shared device health report</li></ul></div>
      <div class="tier"><div class="name">Business</div><div class="amount">¥99 <small>CNY / device / year</small></div><ul><li>Central management console</li><li>Batch deployment</li><li>API integration</li><li>Priority support</li></ul></div>
    </div>
    <p style="margin-top:22px"><a class="btn btn-ghost" href="pricing.html">Compare every tier</a></p>
  </div>
</section>

<section id="roadmap">
  <div class="wrap">
    <div class="sec-head"><span class="sec-tag">ROADMAP</span><h2>Where it is going</h2></div>
    <div class="road">
      <div class="road-row"><div><span class="ver">V1.0</span><span class="when">2026 Q4</span></div><p>Core diagnostics and optimisation, on Windows.</p></div>
      <div class="road-row"><div><span class="ver">V1.5</span><span class="when">2027 Q2</span></div><p>Predictive maintenance, and macOS.</p></div>
      <div class="road-row"><div><span class="ver">V2.0</span><span class="when">2027 Q4</span></div><p>Business edition: central console and API.</p></div>
      <div class="road-row"><div><span class="ver">V2.5</span><span class="when">2028 Q2</span></div><p>Multi-device coordination and cross-device scheduling.</p></div>
      <div class="road-row"><div><span class="ver">V3.0</span><span class="when">2028 Q4</span></div><p>A self-learning system that keeps improving with use.</p></div>
    </div>
  </div>
</section>

<section class="cta">
  <div class="wrap">
    <h2>See what your PC has been trying to tell you.</h2>
    <p>Install it, run one diagnostic, and get an answer in plain language. Fourteen days of everything, no card.</p>
    <a class="btn btn-primary" href="download.html">Download for Windows</a>
    <a class="btn btn-ghost" href="trial.html">Request a business trial</a>
  </div>
</section>
"""

INDEX_ZH = """
<header class="hero">
  <div class="wrap hero-grid">
    <div>
      <span class="eyebrow"><i></i>Windows · V1.0 将于 2026 Q4 发布</span>
      <h1>在电脑变慢<em>之前</em>，先让 AI 照顾它。</h1>
      <p class="lede">
        AI电脑大师在设备本地读取 CPU、内存、磁盘、温度与驱动，用大白话讲清到底哪里出了问题，
        并且只执行你确认过的改动。
      </p>
      <div class="hero-cta">
        <a class="btn btn-primary" href="download.zh.html">下载 Windows 版</a>
        <a class="btn btn-ghost" href="pricing.zh.html">查看价格</a>
      </div>
      <p class="hero-note">14 天全功能试用 · 无需绑定银行卡 · 端侧推理</p>
    </div>

    <div class="panel" aria-hidden="true">
      <div class="panel-top">
        <span class="dot"></span><span class="dot"></span><span class="dot"></span>
        <span class="panel-title">健康度 · 实时</span>
      </div>
      <div class="score">
        <div class="ring"></div>
        <div class="score-meta"><b>系统健康度 86</b><span>3 项发现 · 1 项需要处理</span></div>
      </div>
      <div class="bars">
        <div class="bar-row"><span>CPU</span><span class="track"><span class="fill" style="width:34%"></span></span><span class="val">34%</span></div>
        <div class="bar-row"><span>内存</span><span class="track"><span class="fill warn" style="width:81%"></span></span><span class="val">81%</span></div>
        <div class="bar-row"><span>磁盘</span><span class="track"><span class="fill" style="width:52%"></span></span><span class="val">52%</span></div>
        <div class="bar-row"><span>温度</span><span class="track"><span class="fill" style="width:46%"></span></span><span class="val">46°C</span></div>
      </div>
    </div>
  </div>
</header>

<section id="features">
  <div class="wrap">
    <div class="sec-head">
      <span class="sec-tag">核心能力</span>
      <h2>四件事，全部在你自己的机器上完成</h2>
      <p>所有检测都在本地跑。云端只负责账号与订阅，不负责盯着你的电脑。</p>
    </div>
    <div class="grid-4">
      <div class="card">
        <div class="ico"><svg viewBox="0 0 24 24" fill="none"><circle cx="11" cy="11" r="6.5" stroke="currentColor" stroke-width="1.7"/><path d="M16 16l4 4" stroke="currentColor" stroke-width="1.7" stroke-linecap="round"/></svg></div>
        <h3>智能诊断</h3>
        <p>一键诊断，或直接用自己的话问——"为什么我的电脑变慢了？"给出健康评分、性能瓶颈、根因分析，以及可导出分享的报告。</p>
      </div>
      <div class="card">
        <div class="ico"><svg viewBox="0 0 24 24" fill="none"><path d="M12 3v3M12 18v3M3 12h3M18 12h3M5.6 5.6l2.1 2.1M16.3 16.3l2.1 2.1M18.4 5.6l-2.1 2.1M7.7 16.3l-2.1 2.1" stroke="currentColor" stroke-width="1.6" stroke-linecap="round"/><circle cx="12" cy="12" r="3.2" stroke="currentColor" stroke-width="1.6"/></svg></div>
        <h3>自动优化</h3>
        <p>内存释放、启动项管理、磁盘清理、驱动检查、系统设置优化。默认只读模式，每次改动前自动创建还原点，支持一键回滚。</p>
      </div>
      <div class="card">
        <div class="ico"><svg viewBox="0 0 24 24" fill="none"><path d="M12 3l7 3v6c0 4.2-2.9 7.6-7 9-4.1-1.4-7-4.8-7-9V6l7-3z" stroke="currentColor" stroke-width="1.6" stroke-linejoin="round"/><path d="M12 8v4.5M12 16h.01" stroke="currentColor" stroke-width="1.7" stroke-linecap="round"/></svg></div>
        <h3>预测性维护</h3>
        <p>看的是趋势，不只是当下：温度攀升、磁盘劣化、反复卡顿。在它变成一次维修之前，先提醒你。</p>
      </div>
      <div class="card">
        <div class="ico"><svg viewBox="0 0 24 24" fill="none"><rect x="4.5" y="10" width="15" height="10" rx="2.2" stroke="currentColor" stroke-width="1.6"/><path d="M8 10V7.5a4 4 0 018 0V10" stroke="currentColor" stroke-width="1.6"/></svg></div>
        <h3>隐私与安全</h3>
        <p>除非你主动导出，遥测数据留在设备本地。每一步操作都有完整审计日志，账号支持两步验证。</p>
      </div>
    </div>
  </div>
</section>

<section id="how">
  <div class="wrap">
    <div class="sec-head">
      <span class="sec-tag">工作原理</span>
      <h2>端云协同，四层架构</h2>
      <p>采集与推理都在本地，断网也能继续工作。</p>
    </div>
    <div class="grid-4 flow">
      <div class="card"><h3>数据采集</h3><p>CPU、内存、磁盘、网络、温度、进程、启动项、驱动。正常每 5 秒采样，异常时提升到 500 毫秒。</p></div>
      <div class="card"><h3>AI 推理</h3><p>本地大模型与轻量模型，NPU / GPU 加速。异常检测、根因分析、优化建议，无需往返云端。</p></div>
      <div class="card"><h3>决策执行</h3><p>默认只读。系统修改必须经你确认，操作前自动创建还原点，并且全程可回滚。</p></div>
      <div class="card"><h3>用户交互</h3><p>自然语言对话、可视化仪表盘、诊断报告、优化中心与订阅管理。</p></div>
    </div>
  </div>
</section>

<section id="pricing">
  <div class="wrap">
    <div class="sec-head">
      <span class="sec-tag">价格</span>
      <h2>先免费用。它值了再付。</h2>
      <p>每个新账号都有 14 天全功能试用。到期后，免费版保留基础诊断能力。</p>
    </div>
    <div class="price-grid">
      <div class="tier"><div class="name">试用版</div><div class="amount">免费 <small>14 天</small></div><ul><li>全功能诊断</li><li>基础优化</li><li>无需绑定银行卡</li></ul></div>
      <div class="tier feature"><div class="name">个人版<span class="badge">最受欢迎</span></div><div class="amount">¥199 <small>/ 年</small></div><ul><li>试用版全部能力，长期可用</li><li>自动优化</li><li>预测性维护</li><li>隐私与安全设置</li></ul></div>
      <div class="tier"><div class="name">家庭版</div><div class="amount">¥349 <small>/ 年</small></div><ul><li>个人版全部权益</li><li>最多 5 台设备</li><li>设备健康报告共享</li></ul></div>
      <div class="tier"><div class="name">企业版</div><div class="amount">¥99 <small>/ 设备 / 年</small></div><ul><li>集中管理控制台</li><li>批量部署</li><li>API 集成</li><li>优先支持</li></ul></div>
    </div>
    <p style="margin-top:22px"><a class="btn btn-ghost" href="pricing.zh.html">对比全部版本</a></p>
  </div>
</section>

<section id="roadmap">
  <div class="wrap">
    <div class="sec-head"><span class="sec-tag">路线图</span><h2>接下来要走到哪里</h2></div>
    <div class="road">
      <div class="road-row"><div><span class="ver">V1.0</span><span class="when">2026 Q4</span></div><p>核心诊断与优化上线，Windows 平台。</p></div>
      <div class="road-row"><div><span class="ver">V1.5</span><span class="when">2027 Q2</span></div><p>预测性维护上线，支持 macOS。</p></div>
      <div class="road-row"><div><span class="ver">V2.0</span><span class="when">2027 Q4</span></div><p>企业版发布：集中管理控制台与 API。</p></div>
      <div class="road-row"><div><span class="ver">V2.5</span><span class="when">2028 Q2</span></div><p>多设备协同与跨设备智能调度。</p></div>
      <div class="road-row"><div><span class="ver">V3.0</span><span class="when">2028 Q4</span></div><p>自学习系统，越用越准。</p></div>
    </div>
  </div>
</section>

<section class="cta">
  <div class="wrap">
    <h2>看看你的电脑一直在试图告诉你什么。</h2>
    <p>装上，跑一次诊断，用大白话拿到答案。14 天全功能，无需绑卡。</p>
    <a class="btn btn-primary" href="download.zh.html">下载 Windows 版</a>
    <a class="btn btn-ghost" href="trial.zh.html">申请企业试用</a>
  </div>
</section>
"""


def write(slug, lang, title, desc, body, noindex=False):
    path = OUT / href(lang, slug)
    path.write_text(shell(lang, slug, title, desc, body, noindex=noindex), encoding="utf-8")
    return path


# Bump this when the site's content meaningfully changes (drives sitemap lastmod).
SITE_UPDATED = "2026-09-13"


def write_sitemap():
    """sitemap.xml with per-URL lastmod and hreflang alternates (x-default = English)."""
    lines = [
        '<?xml version="1.0" encoding="UTF-8"?>',
        '<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9"',
        '        xmlns:xhtml="http://www.w3.org/1999/xhtml">',
    ]
    slugs = ["index"] + [s for s in PAGES if s != "404"]
    for slug in slugs:
        for lang in ("en", "zh"):
            lines.append("  <url>")
            lines.append(f"    <loc>{page_url(lang, slug)}</loc>")
            lines.append(f"    <lastmod>{SITE_UPDATED}</lastmod>")
            lines.append(f'    <xhtml:link rel="alternate" hreflang="en" href="{page_url("en", slug)}"/>')
            lines.append(f'    <xhtml:link rel="alternate" hreflang="zh-Hans" href="{page_url("zh", slug)}"/>')
            lines.append(f'    <xhtml:link rel="alternate" hreflang="x-default" href="{page_url("en", slug)}"/>')
            lines.append("  </url>")
    lines.append("</urlset>")
    (OUT / "sitemap.xml").write_text("\n".join(lines) + "\n", encoding="utf-8")


# Crawlers used by generative / answer engines. Explicitly welcomed: when an LLM
# is asked "what is AIPCMaster", these are the bots that go and find out.
AI_CRAWLERS = [
    "GPTBot", "OAI-SearchBot", "ChatGPT-User", "ClaudeBot", "Claude-User",
    "anthropic-ai", "PerplexityBot", "Perplexity-User", "Google-Extended",
    "Applebot-Extended", "CCBot", "Bytespider", "cohere-ai", "meta-externalagent",
    "Amazonbot", "YouBot",
]


def write_robots():
    lines = [
        "# AIPCMaster — https://aipcmaster.com",
        "# All crawlers are welcome, including generative-engine crawlers.",
        "",
        "User-agent: *",
        "Allow: /",
        "Disallow: /404.html",
        "Disallow: /404.zh.html",
        "",
        "# Generative engine crawlers — explicitly welcome",
    ]
    for bot in AI_CRAWLERS:
        lines += [f"User-agent: {bot}", "Allow: /", ""]
    lines.append("Sitemap: https://aipcmaster.com/sitemap.xml")
    (OUT / "robots.txt").write_text("\n".join(lines) + "\n", encoding="utf-8")


def write_humans():
    """humans.txt — who builds this (https://humanstxt.org)."""
    text = f"""/* TEAM */
  Project: AIPCMaster (AI电脑大师)
  Contact: dev@aipcmaster.com
  Support: support@aipcmaster.com
  Security: security@aipcmaster.com
  Business: business@aipcmaster.com
  Location: aipcmaster.com

/* THANKS */
  Everyone who ran an early build and told us what broke.

/* SITE */
  Standards: HTML5, CSS3, Schema.org JSON-LD
  Components: static HTML generated by web-src/generate.py
  Software: Rust, .NET 8 / WPF, Pillow (build-time only)
  Language: English, 简体中文
  Last update: {SITE_UPDATED}
"""
    (OUT / "humans.txt").write_text(text, encoding="utf-8")


def write_llms():
    """llms.txt — a short, citable brief for LLMs (https://llmstxt.org)."""
    text = f"""# AIPCMaster (AI电脑大师)

> AIPCMaster is an AI-powered PC diagnostics and optimization tool for Windows 10 and 11.
> It reads CPU, memory, disk, temperature, process, startup and driver data on the device
> itself, explains in plain language what is actually wrong, and applies only the
> optimizations the user approves. Collection and inference are local; the cloud handles
> only account, device list and subscription. V1.0 ships in Q4 2026.

## Key facts

- Category: system utility / PC optimization software.
- Platform: Windows 10 and Windows 11 (V1.0, Q4 2026). macOS planned for V1.5 (Q2 2027).
- Architecture: edge-cloud. Sampling runs every 5 seconds, or every 500 ms when an anomaly is detected.
- Safety model: read-only by default; every change needs confirmation; a system restore point is created first; every action can be rolled back and is written to an audit log.
- Privacy: telemetry stays on the device unless the user exports it.
- Pricing: 14-day full free trial (no card). Personal ¥199/year, Family ¥349/year (up to 5 devices), Business ¥99 per device per year.
- Languages: English and Simplified Chinese.
- Contact: support@aipcmaster.com, business@aipcmaster.com, security@aipcmaster.com.

## Pages

- [Home]({SITE}/): product overview, capabilities, architecture, pricing summary.
- [Download]({SITE}/download.html): Windows build availability and the notify list.
- [Pricing]({SITE}/pricing.html): full tier comparison.
- [Business]({SITE}/business.html): central console, batch deployment and API integration.
- [Developers]({SITE}/developers.html): API and integration information.
- [Docs]({SITE}/docs.html): documentation centre.
- [Security]({SITE}/security.html): security model and disclosure policy.
- [Support]({SITE}/support.html): help and contact channels.
- [Chinese home]({SITE}/index.zh.html): 中文首页。
"""
    (OUT / "llms.txt").write_text(text, encoding="utf-8")


def with_faq(html, lang):
    """Insert the FAQ section just before the closing CTA block."""
    marker = '<section class="cta">'
    return html.replace(marker, faq_html(lang) + marker, 1) if marker in html else html + faq_html(lang)


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    written = []

    # Homepage: bespoke hero, same shell. FAQ section sits before the final CTA.
    written.append(write(
        "index", "en", "AIPCMaster — AI-powered PC diagnostics and optimization",
        "AIPCMaster watches CPU, memory, disk and drivers on your PC, explains the cause in "
        "plain language, and fixes only what you approve. On-device AI. 14-day trial.",
        with_faq(INDEX_EN, "en"),
    ))
    written.append(write(
        "index", "zh", "AIPCMaster（AI电脑大师）— 端侧 AI 电脑诊断与优化",
        "AI电脑大师在设备本地读取 CPU、内存、磁盘与驱动状态，用大白话讲清问题根源，"
        "只执行你确认过的改动。端侧推理，14 天全功能试用。",
        with_faq(INDEX_ZH, "zh"),
    ))

    for slug, langs in PAGES.items():
        for lang in ("en", "zh"):
            title, desc, body = langs[lang]
            written.append(write(slug, lang, title, desc, body, noindex=(slug == "404")))

    write_sitemap()
    write_robots()
    write_llms()
    write_humans()

    for p in sorted(written):
        print(f"  {p.relative_to(ROOT)}  ({p.stat().st_size} bytes)")
    for extra in ("sitemap.xml", "robots.txt", "llms.txt", "humans.txt"):
        print(f"  web/{extra}")
    print(f"\n{len(written)} pages + sitemap/robots/llms/humans written to {OUT}")


if __name__ == "__main__":
    main()
