"""Renders the Widgets board picker screenshots of the Limvo card.

Usage: python scripts/widget-art.py

Writes src-tauri/msix/widgets/screenshot-light.png and screenshot-dark.png
(300 x 304, the medium widget size the picker shows), drawn like the card
the provider sends: three limits with their pace and reset. Needs Pillow and
Segoe UI, so run it on Windows. The images are committed; run it again only
after the card's layout changes.
"""

from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "src-tauri" / "msix" / "widgets"
FONTS = Path("C:/Windows/Fonts")
SIZE = (300, 304)
SCALE = 2  # Drawn at twice the size, then reduced, for smooth text.

ROWS = [
    ("Claude", "Running hot · Tue 21:00", "78%", True),
    ("Codex", "On track · Mon 09:00", "23%", False),
    ("Copilot", "On track · Dec 1 01:00", "18%", False),
]

THEMES = {
    "light": {
        "background": (243, 243, 243),
        "text": (27, 27, 27),
        "subtle": (93, 93, 93),
        "warning": (157, 93, 0),
        "button": (255, 255, 255),
        "stroke": (0, 0, 0, 18),
    },
    "dark": {
        "background": (44, 44, 44),
        "text": (255, 255, 255),
        "subtle": (207, 207, 207),
        "warning": (242, 201, 76),
        "button": (61, 61, 61),
        "stroke": (255, 255, 255, 20),
    },
}


def font(name: str, size: int) -> ImageFont.FreeTypeFont:
    return ImageFont.truetype(str(FONTS / name), size * SCALE)


def render(theme: dict) -> Image.Image:
    width, height = SIZE[0] * SCALE, SIZE[1] * SCALE
    image = Image.new("RGBA", (width, height), theme["background"])
    draw = ImageDraw.Draw(image)
    s = lambda value: value * SCALE  # noqa: E731

    bold = font("segoeuib.ttf", 14)
    small = font("segoeui.ttf", 12)
    large = font("segoeuib.ttf", 20)

    y = s(18)
    for name, detail, value, hot in ROWS:
        draw.text((s(16), y), name, font=bold, fill=theme["text"])
        draw.text((s(16), y + s(20)), detail, font=small, fill=theme["subtle"])
        right = width - s(16)
        value_width = draw.textlength(value, font=large)
        draw.text(
            (right - value_width, y + s(6)),
            value,
            font=large,
            fill=theme["warning"] if hot else theme["text"],
        )
        y += s(58)

    # The refresh button the larger sizes carry.
    button = (s(16), height - s(52), width - s(16), height - s(18))
    draw.rounded_rectangle(button, radius=s(4), fill=theme["button"], outline=theme["stroke"])
    label = "Refresh"
    label_width = draw.textlength(label, font=small)
    draw.text(
        ((width - label_width) / 2, button[1] + s(8)),
        label,
        font=small,
        fill=theme["text"],
    )
    return image.resize(SIZE, Image.Resampling.LANCZOS)


def main() -> None:
    OUT.mkdir(parents=True, exist_ok=True)
    for name, theme in THEMES.items():
        path = OUT / f"screenshot-{name}.png"
        render(theme).save(path, optimize=True)
        print(f"wrote {path.relative_to(ROOT)}")


if __name__ == "__main__":
    main()
