#!/usr/bin/env python3
"""Fetch public card records and card images for the 3DS-era English set pool.

The artwork stays attributed to Bushiroad and is kept as local prototype assets.
This script does not access game binaries, ROMs, or protected game files.
"""

from __future__ import annotations

import json
import io
import re
import sys
import threading
import time
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path
from urllib.error import HTTPError, URLError
from urllib.parse import quote
from urllib.request import Request, urlopen

try:
    from PIL import Image
except ImportError as error:
    raise SystemExit(
        "Install the asset tooling first: python3 -m pip install -r requirements-assets.txt"
    ) from error


ROOT = Path(__file__).resolve().parents[1]
ASSET_DIR = ROOT / "assets" / "card_images"
DATABASE_PATH = ROOT / "assets" / "official_era_cards.json"
SOURCE_INDEX_PATH = ROOT / "assets" / "official_era_card_sources.csv"

API_BASE = "https://en.cf-vanguard.com/manage/en-api"
IMAGE_BASE = "https://en.cf-vanguard.com/wordpress/wp-content/images/cardlist/"
SOURCE_CARD_LIST = "https://en.cf-vanguard.com/cardlist/"
MAX_IMAGE_EDGE = 512
WORKERS = 4
MIN_REQUEST_INTERVAL = 0.20

SET_PATTERN = re.compile(r"\[(BT|EB|TD)(\d+)\]")
SAFE_FILENAME = re.compile(r"[^A-Za-z0-9_.-]+")


class Pacer:
    """Keep requests to the official card service spaced out across workers."""

    def __init__(self, interval: float) -> None:
        self.interval = interval
        self.lock = threading.Lock()
        self.next_request_at = 0.0

    def wait(self) -> None:
        with self.lock:
            now = time.monotonic()
            if now < self.next_request_at:
                time.sleep(self.next_request_at - now)
            self.next_request_at = time.monotonic() + self.interval


PACER = Pacer(MIN_REQUEST_INTERVAL)


def read_url(url: str) -> bytes:
    request = Request(
        url,
        headers={
            "User-Agent": "VanguardStoryPrototype/0.1 (local personal cardfight prototype)",
            "Accept": "application/json,image/*,*/*;q=0.8",
        },
    )
    last_error: Exception | None = None
    for attempt in range(4):
        PACER.wait()
        try:
            with urlopen(request, timeout=30) as response:
                return response.read()
        except (HTTPError, URLError, TimeoutError, OSError) as error:
            last_error = error
            if attempt < 3:
                time.sleep(1.0 + attempt * 2.0)
    raise RuntimeError(f"Failed to fetch {url}: {last_error}")


def fetch_json(url: str) -> dict:
    return json.loads(read_url(url))


def included_product(product_name: str) -> bool:
    match = SET_PATTERN.search(product_name)
    if not match:
        return False
    kind, number_text = match.groups()
    number = int(number_text)
    return (
        kind == "BT" and 1 <= number <= 15
        or kind == "EB" and 1 <= number <= 10
        or kind == "TD" and 1 <= number <= 13
    )


def fetch_products() -> list[dict]:
    products = fetch_json(f"{API_BASE}/products").get("items", [])
    selected = [product for product in products if included_product(product["name"])]
    return sorted(selected, key=lambda product: int(product["id"]))


def fetch_expansion_cards(expansion_id: str) -> list[dict]:
    cards: list[dict] = []
    page = 1
    per_page = 100
    while True:
        data = fetch_json(
            f"{API_BASE}/cards?expansion={quote(expansion_id)}"
            f"&per_page={per_page}&page={page}"
        )
        cards.extend(data.get("items", []))
        if page * per_page >= int(data.get("total", 0)):
            break
        page += 1
    return cards


def image_filename(card: dict) -> str:
    number = SAFE_FILENAME.sub("_", str(card.get("card_number", "unknown")))
    return f"{number}.png"


def image_url(picture: str) -> str:
    if picture.startswith(("https://", "http://")):
        return picture
    return IMAGE_BASE + quote(picture.lstrip("/"), safe="/._-~")


def download_and_optimize(card: dict) -> tuple[str, str | None]:
    picture = str(card.get("picture", "")).strip()
    if not picture:
        return str(card.get("card_number", "unknown")), None

    destination = ASSET_DIR / image_filename(card)
    if destination.exists() and destination.stat().st_size > 0:
        return str(card.get("card_number", "unknown")), destination.relative_to(ROOT).as_posix()

    image_bytes = read_url(image_url(picture))
    ASSET_DIR.mkdir(parents=True, exist_ok=True)
    try:
        image = Image.open(io.BytesIO(image_bytes))
        image.thumbnail((MAX_IMAGE_EDGE, MAX_IMAGE_EDGE), Image.Resampling.LANCZOS)
        if image.mode in ("RGBA", "LA"):
            rgba = image.convert("RGBA")
            background = Image.new("RGB", rgba.size, "white")
            background.paste(rgba, mask=rgba.getchannel("A"))
            image = background
        else:
            image = image.convert("RGB")
        # Macroquad's default image decoder supports PNG, not JPEG. Palette
        # quantization keeps this complete art set small enough to bundle.
        image.quantize(colors=256, method=Image.Quantize.MEDIANCUT).save(
            destination, format="PNG", optimize=True
        )
    except Exception as error:
        destination.unlink(missing_ok=True)
        raise RuntimeError(f"Image conversion failed for {picture}: {error}") from error
    return str(card.get("card_number", "unknown")), destination.relative_to(ROOT).as_posix()


def write_metadata(products: list[dict], cards: list[dict]) -> None:
    product_names = {str(product["id"]): product["name"] for product in products}
    records = []
    for card in cards:
        record = {
            key: card.get(key)
            for key in (
                "card_number",
                "card_name",
                "card_kind",
                "rare",
                "expansion",
                "tribe",
                "country",
                "clan",
                "grade",
                "power",
                "shield",
                "critical",
                "skill_icon",
                "card_trigger",
                "text",
                "flavor",
                "illustrator",
                "picture",
            )
        }
        record["product"] = product_names.get(str(card.get("expansion", "")), "")
        record["source_url"] = SOURCE_CARD_LIST + "?cardno=" + quote(
            str(card.get("card_number", "")), safe=""
        )
        record["image_file"] = f"assets/card_images/{image_filename(card)}" if card.get("picture") else None
        records.append(record)

    DATABASE_PATH.write_text(
        json.dumps(
            {
                "source": SOURCE_CARD_LIST,
                "scope": "English official card entries in BT01-BT15, EB01-EB10, and TD01-TD13",
                "products": products,
                "cards": records,
            },
            ensure_ascii=False,
            indent=2,
        )
        + "\n",
        encoding="utf-8",
    )

    with SOURCE_INDEX_PATH.open("w", encoding="utf-8", newline="") as output:
        output.write("card_number,card_name,clan,grade,product,image_file,source_url\n")
        for record in records:
            values = [
                record.get("card_number") or "",
                record.get("card_name") or "",
                record.get("clan") or "",
                record.get("grade") or "",
                record.get("product") or "",
                record.get("image_file") or "",
                record.get("source_url") or "",
            ]
            escaped = ['"' + str(value).replace('"', '""') + '"' for value in values]
            output.write(",".join(escaped) + "\n")


def main() -> int:
    ASSET_DIR.mkdir(parents=True, exist_ok=True)
    products = fetch_products()
    print(f"Reading card records from {len(products)} official 3DS-era products...")

    all_cards: dict[str, dict] = {}
    for product in products:
        product_cards = fetch_expansion_cards(str(product["id"]))
        for card in product_cards:
            number = str(card.get("card_number", "")).strip()
            if number:
                all_cards[number] = card
        print(f"  {product['name']}: {len(product_cards)} entries")

    cards = sorted(all_cards.values(), key=lambda card: str(card.get("card_number", "")))
    write_metadata(products, cards)
    print(f"Saved metadata for {len(cards)} distinct card numbers.")

    failures = []
    completed = 0
    with ThreadPoolExecutor(max_workers=WORKERS) as executor:
        futures = [executor.submit(download_and_optimize, card) for card in cards]
        for future in as_completed(futures):
            try:
                _, local_path = future.result()
                if local_path is None:
                    failures.append("missing picture metadata")
            except Exception as error:  # Keep progress and report all failed sources.
                failures.append(str(error))
            completed += 1
            if completed % 50 == 0 or completed == len(futures):
                print(f"  Images processed: {completed}/{len(futures)}")

    if failures:
        print(f"Completed with {len(failures)} missing or failed images:")
        for error in failures[:30]:
            print(f"  - {error}")
        if len(failures) > 30:
            print(f"  ... and {len(failures) - 30} more")
        return 1

    print(f"Saved optimized card images to {ASSET_DIR}.")
    print(f"Metadata: {DATABASE_PATH}")
    print(f"Source index: {SOURCE_INDEX_PATH}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
