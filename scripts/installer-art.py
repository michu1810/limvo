"""Renders the NSIS installer artwork from the app icon.

Usage: npm run art:installer   (or: python scripts/installer-art.py)

Writes src-tauri/installer/sidebar.bmp (164 x 314, welcome and finish pages)
and src-tauri/installer/header.bmp (150 x 57, every other page). Run it after
changing src-tauri/icons/source.svg and `npx tauri icon`. Needs Pillow.
"""

from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

ROOT = Path(__file__).resolve().parent.parent
ICON = ROOT / "src-tauri" / "icons" / "icon.png"
OUT = ROOT / "src-tauri" / "installer"
SCALE = 4

ACCENT = (200, 85, 61)
SURFACE_DARK = (31, 29, 28)
TRACK_DARK = (58, 54, 51)
TEXT_ON_DARK = (255, 255, 255)
MUTED_ON_DARK = (196, 190, 186)
SURFACE_LIGHT = (255, 255, 255)

FONT_DIR = Path("C:/Windows/Fonts")


def font(names: list[str], size: int) -> ImageFont.FreeTypeFont:
    for name in names:
        path = FONT_DIR / name
        if path.exists():
            return ImageFont.truetype(str(path), size * SCALE)
    return ImageFont.load_default(size * SCALE)


def icon(size: int) -> Image.Image:
    return Image.open(ICON).convert("RGBA").resize((size * SCALE, size * SCALE), Image.Resampling.LANCZOS)


def sidebar() -> Image.Image:
    width, height = 164 * SCALE, 314 * SCALE
    image = Image.new("RGBA", (width, height), SURFACE_DARK + (255,))
    draw = ImageDraw.Draw(image)

    # A meter down the left edge, filled about halfway, with the pace tick of the app.
    top, bottom = 56 * SCALE, height - 56 * SCALE
    fill_from = top + (bottom - top) * 0.46
    draw.rounded_rectangle([18 * SCALE, top, 21 * SCALE, bottom], radius=2 * SCALE, fill=TRACK_DARK)
    draw.rounded_rectangle([18 * SCALE, fill_from, 21 * SCALE, bottom], radius=2 * SCALE, fill=ACCENT)
    tick = top + (bottom - top) * 0.3
    draw.rounded_rectangle([15 * SCALE, tick - SCALE, 24 * SCALE, tick + SCALE], radius=SCALE, fill=TEXT_ON_DARK)

    image.alpha_composite(icon(64), (34 * SCALE, 60 * SCALE))
    title = font(["SegUIVar.ttf", "segoeuib.ttf"], 21)
    body = font(["SegUIVar.ttf", "segoeui.ttf"], 10)
    draw.text((38 * SCALE, 140 * SCALE), "Headroom", font=title, fill=TEXT_ON_DARK)
    for i, line in enumerate(["Plan limits and pace,", "right in your taskbar."]):
        draw.text((39 * SCALE, (172 + i * 15) * SCALE), line, font=body, fill=MUTED_ON_DARK)
    return image.convert("RGB").resize((164, 314), Image.Resampling.LANCZOS)


def header() -> Image.Image:
    width, height = 150 * SCALE, 57 * SCALE
    image = Image.new("RGBA", (width, height), SURFACE_LIGHT + (255,))
    image.alpha_composite(icon(44), (98 * SCALE, 6 * SCALE))
    return image.convert("RGB").resize((150, 57), Image.Resampling.LANCZOS)


def main() -> None:
    OUT.mkdir(parents=True, exist_ok=True)
    sidebar().save(OUT / "sidebar.bmp")
    header().save(OUT / "header.bmp")
    print(f"wrote {OUT / 'sidebar.bmp'} and {OUT / 'header.bmp'}")


if __name__ == "__main__":
    main()
