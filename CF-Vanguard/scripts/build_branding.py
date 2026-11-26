#!/usr/bin/env python3
"""Composite the app's branding from credited non-AI artwork only.

Requires requirements-assets.txt. No model, generated landscape, or game UI
screenshot is used. Source artwork remains unchanged; outputs are reproducible.
"""
from __future__ import annotations

import hashlib
import json
from pathlib import Path
import shutil
import subprocess

from PIL import Image, ImageChops, ImageDraw, ImageFilter, ImageFont, ImageOps

ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "assets/branding"
RESAMPLE = Image.Resampling.LANCZOS


def artwork(relative: str) -> Image.Image:
    return Image.open(ROOT / relative).convert("RGBA")


def fitted(image: Image.Image, size: tuple[int, int]) -> Image.Image:
    return ImageOps.contain(image, size, RESAMPLE)


def place(canvas: Image.Image, image: Image.Image, xy: tuple[int, int], shadow: int = 0) -> None:
    if shadow:
        shade = Image.new("RGBA", canvas.size)
        mask = Image.new("L", canvas.size)
        mask.paste(image.getchannel("A"), (xy[0], xy[1] + shadow // 3))
        shade.putalpha(mask.filter(ImageFilter.GaussianBlur(shadow)))
        canvas.alpha_composite(shade)
    canvas.alpha_composite(image, xy)


def english_wordmark() -> Image.Image:
    image = artwork("assets/branding/sources/vanguard-wordmark.png")
    # Remove only the original Japanese subtitle, retaining the complete English V/wordmark.
    ImageDraw.Draw(image).rectangle((118, 210, image.width, image.height), fill=(0, 0, 0, 0))
    return image.crop(image.getbbox())


def app_icon() -> Image.Image:
    canvas = Image.new("RGBA", (1024, 1024))
    # Circular era battlefield medallion, not an opaque square app tile.
    face = Image.new("RGBA", canvas.size)
    draw = ImageDraw.Draw(face)
    draw.ellipse((56, 36, 968, 948), fill=(7, 20, 38, 255))
    draw.ellipse((70, 50, 954, 934), outline=(91, 159, 199, 255), width=6)
    rings = artwork("assets/tabletop/vanguard-zones.png").crop((690, 245, 1040, 595)).resize((930, 930), RESAMPLE)
    ring_layer = Image.new("RGBA", canvas.size)
    ring_layer.alpha_composite(rings, (47, 27))
    ring_clip = Image.new("L", canvas.size)
    ImageDraw.Draw(ring_clip).ellipse((56, 36, 968, 948), fill=255)
    ring_layer.putalpha(ImageChops.multiply(ring_layer.getchannel("A"), ring_clip))
    face.alpha_composite(ring_layer)
    canvas.alpha_composite(face)
    characters = Image.new("RGBA", canvas.size)
    aichi = artwork("assets/fighters/aichi-standing.png").crop((86, 0, 380, 430))
    kai = artwork("assets/fighters/kai-standing.png").crop((85, 0, 435, 430))
    place(characters, kai.resize((468, 575), RESAMPLE), (471, 83))
    place(characters, aichi.resize((414, 605), RESAMPLE), (155, 62))
    clip = Image.new("L", canvas.size)
    ImageDraw.Draw(clip).ellipse((65, 38, 959, 948), fill=255)
    characters.putalpha(ImageChops.multiply(characters.getchannel("A"), clip))
    canvas.alpha_composite(characters)
    for path, xy in [("assets/clans/royal-paladin.png", (87, 352)), ("assets/clans/kagero.png", (803, 352))]:
        emblem = fitted(artwork(path), (136, 136))
        place(canvas, emblem, xy, 7)
    logo = fitted(english_wordmark(), (940, 440))
    place(canvas, logo, ((1024 - logo.width) // 2, 583), 16)
    return canvas


def game_logo() -> Image.Image:
    canvas = Image.new("RGBA", (1600, 800))
    mark = fitted(english_wordmark(), (1510, 650))
    place(canvas, mark, ((1600 - mark.width) // 2, 12), 10)
    font = ImageFont.truetype(str(ROOT / "assets/fonts/Rajdhani-SemiBold.ttf"), 78)
    label = "LIMIT BREAK / BREAK RIDE"
    draw = ImageDraw.Draw(canvas)
    box = draw.textbbox((0, 0), label, font=font)
    x = (1600 - (box[2] - box[0])) // 2
    draw.text((x, 690), label, font=font, fill=(244, 211, 133), stroke_width=3, stroke_fill=(4, 16, 29))
    for path, x in [("assets/clans/royal-paladin.png", 115), ("assets/clans/kagero.png", 1405)]:
        emblem = fitted(artwork(path), (80, 80))
        place(canvas, emblem, (x, 687))
    return canvas


def export_icons(master: Image.Image) -> None:
    iconset = OUTPUT / "AppIcon.iconset"
    iconset.mkdir(exist_ok=True)
    for size in [16, 32, 64, 128, 256, 512, 1024]:
        image = master.resize((size, size), RESAMPLE)
        image.save(OUTPUT / f"app-icon-{size}.png", optimize=True)
        if size <= 64:
            (OUTPUT / f"app-icon-{size}.rgba").write_bytes(image.tobytes())
    for points in [16, 32, 128, 256, 512]:
        for density in [1, 2]:
            pixels = points * density
            suffix = "@2x" if density == 2 else ""
            master.resize((pixels, pixels), RESAMPLE).save(iconset / f"icon_{points}x{points}{suffix}.png", optimize=True)
    master.save(OUTPUT / "app-icon.ico", sizes=[(n, n) for n in [16, 32, 48, 64, 128, 256]])
    if shutil.which("iconutil"):
        subprocess.run(["iconutil", "-c", "icns", str(iconset), "-o", str(OUTPUT / "AppIcon.icns")], check=True)
    else:
        master.save(OUTPUT / "AppIcon.icns", format="ICNS")


def manifest() -> None:
    inputs = [
        ("assets/branding/sources/vanguard-wordmark.png", "https://en.cf-vanguard.com/business/media-kit/logos/", "Official media-kit wordmark; Japanese subtitle omitted from the English output"),
        ("assets/fighters/aichi-standing.png", "https://anime.cf-vanguard.com/vg2018-if/characters/aichi-sendou/", "Official Aichi render; upper-body crop and resize"),
        ("assets/fighters/kai-standing.png", "https://anime.cf-vanguard.com/vg2018-if/characters/toshiki-kai/", "Official Kai render; upper-body crop and resize"),
        ("assets/tabletop/vanguard-zones.png", "https://www.reddit.com/r/cardfightvanguard/comments/rpbpwi/", "Collected community mat: Vanguard circle crop"),
    ]
    clans = json.loads((ROOT / "assets/clans/sources.json").read_text())
    for clan in ["Royal Paladin", "Kagero"]:
        source = next(r for r in clans if r["clan"] == clan)
        inputs.append((source["file"], source["source_url"], "Original wiki clan symbol; resize only"))
    data = {"method": "Deterministic Pillow compositing of non-AI source artwork, circle geometry, shadows, and licensed typography. No image generation or AI assets.",
            "reference_game": "assets/branding/sources/lock-on-victory-logo.png (publisher layout reference only)",
            "inputs": [{"file": f, "source": url, "use": use, "sha256": hashlib.sha256((ROOT / f).read_bytes()).hexdigest()} for f, url, use in inputs],
            "font": {"file": "assets/fonts/Rajdhani-SemiBold.ttf", "license": "assets/fonts/OFL.txt"},
            "outputs": {p.name: hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(OUTPUT.iterdir()) if p.is_file() and p.suffix in {".png", ".rgba", ".icns", ".ico"}}}
    (OUTPUT / "provenance.json").write_text(json.dumps(data, indent=2) + "\n")


def main() -> None:
    OUTPUT.mkdir(exist_ok=True)
    icon = app_icon()
    icon.save(OUTPUT / "app-icon.png", optimize=True)
    game_logo().save(OUTPUT / "game-logo.png", optimize=True)
    export_icons(icon)
    manifest()
    print("Built transparent app-icon.png, game-logo.png, PNG/ICO/ICNS icon sizes, and provenance.json.")


if __name__ == "__main__":
    main()
