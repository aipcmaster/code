#!/usr/bin/env python3
"""Generate Open Graph / Twitter social preview images (1200x630).

One image per page per language, plus the homepage default:
  web/og-image.png          web/og-image.zh.png          (homepage)
  web/og-<slug>.png         web/og-<slug>.zh.png         (every other page)

Requires Pillow. Run:  python3 make_og_image.py
"""
import pathlib
import sys

from PIL import Image, ImageDraw, ImageFilter, ImageFont

sys.path.insert(0, str(pathlib.Path(__file__).parent))
from content import PAGES  # noqa: E402

ROOT = pathlib.Path(__file__).parent.parent
OUT = ROOT / "web"

W, H = 1200, 630
BG = (10, 12, 15)
ACCENT = (47, 224, 168)
TEXT = (233, 237, 243)
DIM = (152, 162, 179)
FAINT = (102, 112, 133)

LATIN_BOLD = "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf"
LATIN = "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf"
CJK_BOLD = "/usr/share/fonts/opentype/noto/NotoSansCJK-Bold.ttc"

# Short descriptor shown under the page title. Kept terse so it reads at thumbnail size.
DESCRIPTORS = {
    "download": {"en": "Get the Windows build", "zh": "获取 Windows 版本"},
    "pricing": {"en": "Plans, tiers and pricing", "zh": "版本、档位与价格"},
    "business": {"en": "For teams and IT", "zh": "面向团队与 IT 部门"},
    "developers": {"en": "API and integrations", "zh": "API 与集成"},
    "security": {"en": "Security model and disclosure", "zh": "安全模型与漏洞披露"},
    "docs": {"en": "Documentation centre", "zh": "文档中心"},
    "trial": {"en": "Request a business trial", "zh": "申请企业试用"},
    "support": {"en": "Help and contact", "zh": "帮助与联系方式"},
    "privacy": {"en": "Privacy policy", "zh": "隐私政策"},
    "terms": {"en": "Terms of service", "zh": "用户协议"},
}

# Homepage headline (index has no PAGES entry).
HOME = {
    "en": ("Your PC, looked after by AI", "before it slows down.",
           "On-device diagnostics · root-cause analysis · one-click rollback"),
    "zh": ("在电脑变慢之前", "先让 AI 照顾它。", "端侧诊断 · 根因分析 · 一键回滚"),
}


def font(path, size, index=0):
    return ImageFont.truetype(path, size, index=index)


def glow(size, center, radius, color, alpha):
    layer = Image.new("RGBA", size, (0, 0, 0, 0))
    d = ImageDraw.Draw(layer)
    x, y = center
    d.ellipse([x - radius, y - radius, x + radius, y + radius], fill=color + (alpha,))
    return layer.filter(ImageFilter.GaussianBlur(radius // 2))


def draw_mark(d, x, y, scale=1.0):
    """The monitor-with-pulse logo mark, matching favicon.svg."""
    w, h = int(72 * scale), int(54 * scale)
    d.rounded_rectangle([x, y, x + w, y + h], radius=int(9 * scale), outline=ACCENT,
                        width=max(2, int(3 * scale)))
    pts = [
        (x + int(14 * scale), y + int(30 * scale)),
        (x + int(24 * scale), y + int(30 * scale)),
        (x + int(32 * scale), y + int(16 * scale)),
        (x + int(42 * scale), y + int(42 * scale)),
        (x + int(50 * scale), y + int(30 * scale)),
        (x + int(60 * scale), y + int(30 * scale)),
    ]
    d.line(pts, fill=TEXT, width=max(2, int(3 * scale)), joint="curve")
    d.line([(x + int(28 * scale), y + h + int(8 * scale)), (x + int(46 * scale), y + h + int(8 * scale))],
           fill=ACCENT, width=max(2, int(3 * scale)))
    d.line([(x + int(37 * scale), y + h), (x + int(37 * scale), y + h + int(8 * scale))],
           fill=ACCENT, width=max(2, int(3 * scale)))


def build(lang, headline, headline_accent, subtitle, out_name):
    img = Image.new("RGB", (W, H), BG)
    img = Image.alpha_composite(img.convert("RGBA"), glow((W, H), (1010, 120), 420, ACCENT, 26))
    img = Image.alpha_composite(img, glow((W, H), (120, 600), 360, (80, 140, 255), 18)).convert("RGB")
    d = ImageDraw.Draw(img)

    d.rounded_rectangle([72, 72, W - 72, H - 72], radius=22, outline=(255, 255, 255, 20), width=1)

    # brand row
    draw_mark(d, 112, 116)
    d.text((212, 108), "AIPCMaster", font=font(LATIN_BOLD, 62), fill=TEXT)
    d.text((214, 182), "AI电脑大师", font=font(CJK_BOLD, 30), fill=ACCENT)

    d.line([(112, 250), (208, 250)], fill=ACCENT, width=4)

    # headline (one or two lines)
    f_head = font(CJK_BOLD, 42, index=0) if lang == "zh" else font(LATIN_BOLD, 44)
    d.text((112, 280), headline, font=f_head, fill=TEXT)
    if headline_accent:
        d.text((112, 336), headline_accent, font=f_head, fill=ACCENT)
        sub_y = 414
    else:
        sub_y = 356

    f_sub = font(CJK_BOLD, 25, index=0) if lang == "zh" else font(LATIN, 27)
    d.text((112, sub_y), subtitle, font=f_sub, fill=DIM)

    # footer
    d.line([(112, 488), (W - 112, 488)], fill=(255, 255, 255, 26), width=1)
    d.text((112, 506), "aipcmaster.com", font=font(LATIN_BOLD, 28), fill=ACCENT)
    tag = "Windows · V1.0 Q4 2026" if lang == "en" else "Windows · V1.0 2026 Q4"
    f_tag = font(LATIN, 22)
    d.text((W - 112 - d.textlength(tag, font=f_tag), 512), tag, font=f_tag, fill=FAINT)

    # PNG-8 quantization: ~45% smaller than truecolor with no visible loss on this
    # flat, dark palette (the only gradient is a soft glow, which 256 colours hold).
    (img.convert("RGB")
        .quantize(colors=256, method=Image.Quantize.MEDIANCUT, dither=Image.Dither.NONE)
        .save(OUT / out_name, "PNG", optimize=True))
    return out_name


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    written = []

    for lang in ("en", "zh"):
        h, ha, sub = HOME[lang]
        written.append(build(lang, h, ha, sub, "og-image.png" if lang == "en" else "og-image.zh.png"))

    for slug in PAGES:
        if slug == "404":
            continue
        for lang in ("en", "zh"):
            title = PAGES[slug][lang][0]
            desc = DESCRIPTORS.get(slug, {}).get(lang, "")
            name = f"og-{slug}.png" if lang == "en" else f"og-{slug}.zh.png"
            written.append(build(lang, title, None, desc, name))

    total = 0
    for name in written:
        size = (OUT / name).stat().st_size
        total += size
        print(f"  web/{name}  ({size:,} bytes)")
    print(f"\n{len(written)} images, {total:,} bytes total")


if __name__ == "__main__":
    main()
