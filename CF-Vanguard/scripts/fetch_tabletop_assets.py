#!/usr/bin/env python3
"""Fetch real artwork and package it for the native tabletop; never generate art.

Install requirements-tabletop-assets.txt, then run from any working directory.
The editable sleeve SVGs use the prototype's rounded card-back geometry around
the complete official product illustration. Only the exterior corners are clear.
"""

from __future__ import annotations

import base64
import hashlib
import io
import json
from pathlib import Path
import urllib.request

import cairosvg
from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
ANIME = "https://anime.cf-vanguard.com/vg2018-if/wordpress/wp-content/uploads/"
ANIME_PAGE = "https://anime.cf-vanguard.com/vg2018-if/characters/aichi-sendou/"
ASSETS = [
    ("fighters/aichi-icon.png", ANIME + "char-icon-01.png", ANIME_PAGE, "png"),
    ("fighters/kai-icon.png", ANIME + "char-icon-02.png", ANIME_PAGE, "png"),
    ("fighters/aichi-standing.png", ANIME + "char-stand-01.png", ANIME_PAGE, "png"),
    ("fighters/kai-standing.png", ANIME + "char-stand-02.png",
     "https://anime.cf-vanguard.com/vg2018-if/characters/toshiki-kai/", "png"),
    ("presentation/vanguard-logo.png",
     "https://en.cf-vanguard.com/wordpress/wp-content/themes/cf-vanguard/img/vg_logo_en.png",
     "https://en.cf-vanguard.com/", "png"),
    ("tabletop/aichi-sleeve.png", "https://cf-vanguard.com/wordpress/wp-content/uploads/sleeve399.jpg",
     "https://cf-vanguard.com/products/supply/minisleeve_399/", "sleeve"),
    ("tabletop/kai-sleeve.png", "https://cf-vanguard.com/wordpress/wp-content/uploads/sleeve389.jpg",
     "https://cf-vanguard.com/products/supply/minisleeve_389/", "sleeve"),
    ("tabletop/vanguard-zones.png", "https://drive.google.com/uc?export=download&id=1_HDsYJEwk1H1S_rg74zXzGthCbwFcUYF",
     "https://www.reddit.com/r/cardfightvanguard/comments/rpbpwi/", "psd"),
    ('fighters/morikawa-standing.png', 'https://anime.cf-vanguard.com/vg2018-if/wordpress/wp-content/uploads/char-stand-09.png', 'https://anime.cf-vanguard.com/vg2018-if/characters/katsumi-morikawa/', 'png'),
    ('fighters/morikawa-icon.png', 'https://anime.cf-vanguard.com/vg2018-if/wordpress/wp-content/uploads/char-icon-09.png', 'https://anime.cf-vanguard.com/vg2018-if/characters/katsumi-morikawa/', 'png'),
    ('fighters/kamui-standing.png', 'https://anime.cf-vanguard.com/vg2018-if/wordpress/wp-content/uploads/char-stand-04.png', 'https://anime.cf-vanguard.com/vg2018-if/characters/kamui-katsuragi/', 'png'),
    ('fighters/kamui-icon.png', 'https://anime.cf-vanguard.com/vg2018-if/wordpress/wp-content/uploads/char-icon-04.png', 'https://anime.cf-vanguard.com/vg2018-if/characters/kamui-katsuragi/', 'png'),
    ('fighters/misaki-standing.png', 'https://anime.cf-vanguard.com/vg2018-if/wordpress/wp-content/uploads/char-stand-03.png', 'https://anime.cf-vanguard.com/vg2018-if/characters/misaki-tokura/', 'png'),
    ('fighters/misaki-icon.png', 'https://anime.cf-vanguard.com/vg2018-if/wordpress/wp-content/uploads/char-icon-03.png', 'https://anime.cf-vanguard.com/vg2018-if/characters/misaki-tokura/', 'png'),
    ('fighters/ren-standing.png', 'https://anime.cf-vanguard.com/vg2018-if/wordpress/wp-content/uploads/char-stand-05.png', 'https://anime.cf-vanguard.com/vg2018-if/characters/ren-suzugamori/', 'png'),
    ('fighters/ren-icon.png', 'https://anime.cf-vanguard.com/vg2018-if/wordpress/wp-content/uploads/char-icon-05.png', 'https://anime.cf-vanguard.com/vg2018-if/characters/ren-suzugamori/', 'png'),
    ('fighters/asaka-standing.png', 'https://anime.cf-vanguard.com/vg2018-if/wordpress/wp-content/uploads/char-stand-06.png', 'https://anime.cf-vanguard.com/vg2018-if/characters/asaka-narumi/', 'png'),
    ('fighters/asaka-icon.png', 'https://anime.cf-vanguard.com/vg2018-if/wordpress/wp-content/uploads/char-icon-06.png', 'https://anime.cf-vanguard.com/vg2018-if/characters/asaka-narumi/', 'png'),
    ('fighters/leon-standing.png', 'https://anime.cf-vanguard.com/vg2018-if/wordpress/wp-content/uploads/char-stand-16.png', 'https://anime.cf-vanguard.com/vg2018-if/characters/leon-souryu/', 'png'),
    ('fighters/leon-icon.png', 'https://anime.cf-vanguard.com/vg2018-if/wordpress/wp-content/uploads/char-icon-16.png', 'https://anime.cf-vanguard.com/vg2018-if/characters/leon-souryu/', 'png'),
    ('fighters/naoki-standing.png', 'https://anime.cf-vanguard.com/vg2018-if/wordpress/wp-content/uploads/char-stand-b02.png', 'https://anime.cf-vanguard.com/vg2018-if/characters/naoki-ishida-hs/', 'png'),
    ('fighters/naoki-icon.png', 'https://anime.cf-vanguard.com/vg2018-if/wordpress/wp-content/uploads/char-icon-b02.png', 'https://anime.cf-vanguard.com/vg2018-if/characters/naoki-ishida-hs/', 'png'),
    ('fighters/takuto-standing.png', 'https://anime.cf-vanguard.com/vg2018-if/wordpress/wp-content/uploads/char-stand-b16.png', 'https://anime.cf-vanguard.com/vg2018-if/characters/takuto-tatsunagi-hs/', 'png'),
    ('fighters/takuto-icon.png', 'https://anime.cf-vanguard.com/vg2018-if/wordpress/wp-content/uploads/char-icon-b16.png', 'https://anime.cf-vanguard.com/vg2018-if/characters/takuto-tatsunagi-hs/', 'png'),
]


def main() -> None:
    records = []
    for filename, url, page, kind in ASSETS:
        request = urllib.request.Request(url, headers={"User-Agent": "VanguardPrototype/0.1 asset-fetch"})
        with urllib.request.urlopen(request, timeout=45) as response:
            data = response.read()
        path = ROOT / "assets" / filename
        path.parent.mkdir(parents=True, exist_ok=True)
        image = Image.open(io.BytesIO(data))  # Reject HTML and unavailable downloads.
        if kind == "sleeve":
            width, height = image.size
            encoded = base64.b64encode(data).decode("ascii")
            # Code-native rounded card frame, consistent with card-back.svg.
            # The artwork remains a single embedded, unmodified downloaded JPEG.
            svg = f'''<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="{width}" height="{height}" viewBox="0 0 {width} {height}">
  <title>{path.stem.replace('-', ' ').title()}</title>
  <defs><clipPath id="sleeve"><rect width="{width}" height="{height}" rx="8"/></clipPath></defs>
  <image width="{width}" height="{height}" clip-path="url(#sleeve)" xlink:href="data:image/jpeg;base64,{encoded}"/>
</svg>'''
            path.with_suffix(".svg").write_text(svg)
            cairosvg.svg2png(bytestring=svg.encode(), write_to=str(path))
        elif kind == "psd":
            # Creator's download is a PSD despite being described as a PNG.
            image.convert("RGBA").save(path)
        else:
            if image.format != "PNG":
                raise ValueError(f"Expected PNG for {filename}")
            path.write_bytes(data)
        result = Image.open(path).convert("RGBA")
        alpha = result.getextrema()[3]
        if alpha != (0, 255):
            raise ValueError(f"Missing transparent/opaque pixels: {filename}: {alpha}")
        records.append({"file": filename, "source_url": url, "source_page": page,
                        "download_sha256": hashlib.sha256(data).hexdigest(),
                        "png_sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
                        "size": list(result.size), "alpha_range": list(alpha),
                        "processing": kind})
        print(f"{filename}: {result.width} x {result.height}, verified transparency")
    (ROOT / "assets/tabletop/sources.json").write_text(json.dumps(records, indent=2) + "\n")


if __name__ == "__main__":
    main()
