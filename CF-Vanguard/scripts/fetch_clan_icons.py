#!/usr/bin/env python3
"""Download the wiki's clan symbols without altering their original artwork.

Uses MediaWiki's filename-hash image paths, as in the user-supplied Kagero URL.
Install requirements-assets.txt first. All sources and hashes are recorded.
"""
from concurrent.futures import ThreadPoolExecutor
import hashlib
import io
import json
from pathlib import Path
from urllib.parse import quote
from urllib.request import Request, urlopen

from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
DESTINATION = ROOT / "assets" / "clans"


def fetch(clan: str) -> dict:
    filename = "Icon_" + clan.replace(" ", "") + ".png"
    digest = hashlib.md5(filename.encode()).hexdigest()
    url = f"https://static.wikia.nocookie.net/cardfight/images/{digest[0]}/{digest[:2]}/{quote(filename)}/revision/latest?format=original"
    request = Request(url, headers={"User-Agent": "CFVanguardPrototype/0.1 asset-fetch"})
    with urlopen(request, timeout=30) as response:
        data = response.read()
    image = Image.open(io.BytesIO(data))
    if image.format != "PNG":
        raise ValueError(f"Expected a PNG for {clan}")
    path = DESTINATION / (clan.lower().replace(" ", "-") + ".png")
    path.write_bytes(data)
    print(f"{clan}: {image.width} x {image.height}", flush=True)
    return {"clan": clan, "file": str(path.relative_to(ROOT)), "source_url": url,
            "source_page": "https://cardfight.fandom.com/wiki/File:" + quote(filename),
            "sha256": hashlib.sha256(data).hexdigest(), "size": list(image.size),
            "processing": "Original PNG, unchanged."}


def main() -> None:
    cards = json.loads((ROOT / "assets/official_era_cards.json").read_text())["cards"]
    clans = sorted({card["clan"] for card in cards})
    DESTINATION.mkdir(parents=True, exist_ok=True)
    with ThreadPoolExecutor(max_workers=6) as pool:
        records = list(pool.map(fetch, clans))
    (DESTINATION / "sources.json").write_text(json.dumps(records, indent=2) + "\n")


if __name__ == "__main__":
    main()
