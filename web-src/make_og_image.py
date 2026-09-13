#!/usr/bin/env python3
"""Generate the Open Graph / Twitter social preview images (1200x630).

Two variants, one per language, so a shared link renders in the reader's language:
  web/og-image.png      (English)
  web/og-image.zh.png   (Chinese)

Requires Pillow. Run:  python3 make_og_image.py
"""
import pathlib

from PIL import Image, ImageDraw, ImageFilter, ImageFont

ROOT = pathlib.Path(__file__).parent.parent
OUT = ROOT / "web"

W, H = 1200, 630
BG = (10, 12, 15)
CARD = (19, 24, 32)
ACCENT = (47, 224, 168)
TEXT = (233, 237, 243)
DIM = (152, 162, 179)
FAINT = (102, 112, 133)

LATIN_BOLD = "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf"
LATIN = "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf"
CJK_BOLD = "/usr/share/fonts/opentype/noto/NotoSansCJK-Bold.ttc"


def font(path, size, index=0):
    return ImageFont.truetype(path, size, index=index)


def glow(size, center, radius, color, alpha):
    """Soft radial glow layer."""
    layer = Image.new("RGBA", size, (0, 0, 0, 0))
    d = ImageDraw.Draw(layer)
    x, y = center
    d.ellipse([x - radius, y - radius, x + radius, y + radius], fill=color + (alpha,))
    return layer.filter(ImageFilter.GaussianBlur(radius // 2))


def draw_mark(d, x, y, scale=1.0):
    """The monitor-with-pulse logo mark, drawn to match favicon.svg."""
    w, h = int(72 * scale), int(54 * scale)
    d.rounded_rectangle([x, y, x + w, y + h], radius=int(9 * scale), outline=ACCENT, width=max(2, int(3 * scale)))
    # pulse line
    pts = [
        (x + int(14 * scale), y + int(30 * scale)),
        (x + int(24 * scale), y + int(30 * scale)),
        (x + int(32 * scale), y + int(16 * scale)),
        (x + int(42 * scale), y + int(42 * scale)),
        (x + int(50 * scale), y + int(30 * scale)),
        (x + int(60 * scale), y + int(30 * scale)),
    ]
    d.line(pts, fill=TEXT, width=max(2, int(3 * scale)), joint="curve")
    # stand
    d.line([(x + int(28 * scale), y + h + int(8 * scale)), (x + int(46 * scale), y + h + int(8 * scale))],
           fill=ACCENT, width=max(2, int(3 * scale)))
    d.line([(x + int(37 * scale), y + h), (x + int(37 * scale), y + h + int(8 * scale))],
           fill=ACCENT, width=max(2, int(3 * scale)))


def build(lang):
    img = Image.new("RGB", (W, H), BG)

    # accent glow, top-right
    g1 = glow((W, H), (1010, 120), 420, ACCENT, 26)
    # second faint glow bottom-left
    g2 = glow((W, H), (120, 600), 360, (80, 140, 255), 18)
    img = Image.alpha_composite(img.convert("RGBA"), g1)
    img = Image.alpha_composite(img, g2).convert("RGB")

    d = ImageDraw.Draw(img)

    # card panel behind the mark/wordmark
    d.rounded_rectangle([72, 72, W - 72, H - 72], radius=22, outline=(255, 255, 255, 20), width=1,
                        fill=None)

    # logo + wordmark
    draw_mark(d, 112, 116, scale=1.0)
    f_word = font(LATIN_BOLD, 62)
    d.text((212, 108), "AIPCMaster", font=f_word, fill=TEXT)

    # Chinese name (both variants carry it, sized to language)
    f_cjk = font(CJK_BOLD, 30, index=0)
    d.text((214, 182), "AI电脑大师", font=f_cjk, fill=ACCENT)

    # accent rule
    d.line([(112, 250), (112 + 96, 250)], fill=ACCENT, width=4)

    # headline
    f_head = font(LATIN_BOLD, 44) if lang == "en" else font(CJK_BOLD, 42, index=0)
    if lang == "en":
        d.text((112, 280), "Your PC, looked after by AI", font=f_head, fill=TEXT)
        d.text((112, 336), "before it slows down.", font=f_head, fill=ACCENT)
        f_sub = font(LATIN, 27)
        d.text((112, 414), "On-device diagnostics · root-cause analysis · one-click rollback",
               font=f_sub, fill=DIM)
    else:
        d.text((112, 280), "在电脑变慢之前", font=f_head, fill=TEXT)
        d.text((112, 336), "先让 AI 照顾它。", font=f_head, fill=ACCENT)
        f_sub = font(CJK_BOLD, 25, index=0)
        d.text((112, 416), "端侧诊断 · 根因分析 · 一键回滚", font=f_sub, fill=DIM)

    # footer strip
    d.line([(112, 488), (W - 112, 488)], fill=(255, 255, 255, 26), width=1)
    f_url = font(LATIN_BOLD, 28)
    d.text((112, 506), "aipcmaster.com", font=f_url, fill=ACCENT)
    f_tag = font(LATIN, 22)
    tag = "Windows · V1.0 Q4 2026" if lang == "en" else "Windows · V1.0 2026 Q4"
    tw = d.textlength(tag, font=f_tag)
    d.text((W - 112 - tw, 512), tag, font=f_tag, fill=FAINT)

    name = "og-image.png" if lang == "en" else "og-image.zh.png"
    img.save(OUT / name, "PNG", optimize=True)
    return name


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    for lang in ("en", "zh"):
        name = build(lang)
        p = OUT / name
        print(f"  web/{name}  ({p.stat().st_size} bytes)")


if __name__ == "__main__":
    main()
