# Assets and data

`card-back.svg`, `card-frame.svg`, and `playmat.svg` are original geometric vector designs created for this prototype. They do not include official card art, logos, screenshots, or character portraits.

`supplied-playmat.png` preserves the earlier supplied match artwork as a reference.

## Downloaded tabletop artwork

- Morikawa, Kamui, Misaki, Ren, Asaka, Leon, Naoki and Takuto now also have unchanged transparent `fighters/<name>-icon.png` and `fighters/<name>-standing.png` downloads from their official anime character pages. Icons are 144 × 144; standing PNGs are 544 × 624. The battle HUD matches the opponent name to its portrait, including Reversed story rivals. The source manifest records the exact individual pages, alpha range and checksums; the fetch script reproduces all downloads.
- `fighters/aichi-icon.png` and `fighters/kai-icon.png` are unchanged transparent PNGs from the [official anime character page](https://anime.cf-vanguard.com/vg2018-if/characters/aichi-sendou/) (`char-icon-01.png` and `char-icon-02.png`).
- `tabletop/aichi-sleeve.png` uses the official [Mini Vol.399 Aichi Part.4](https://cf-vanguard.com/products/supply/minisleeve_399/) illustration. `tabletop/kai-sleeve.png` uses [Mini Vol.389 Kai Part.4](https://cf-vanguard.com/products/supply/minisleeve_389/). The downloads are JPEGs; companion editable SVGs embed those original bytes within rounded card frames, then rasterize to PNG with transparent exterior corners. The printed sleeve artwork stays opaque, as a card back should.
- `tabletop/vanguard-zones.png` is the transparent composite of AllTheCoins' [community playmat template](https://www.reddit.com/r/cardfightvanguard/comments/rpbpwi/), shared by its creator for custom mats. The linked download is actually a PSD, so the fetcher exports its existing composite to RGBA PNG. This is a community template, not an official Bushiroad playmat. The match samples its circles and places the auxiliary zones according to the classic official guide; the full template's overDress logo, Order Zone and Ride Deck are not drawn.
- `tabletop/sources.json` records direct download URLs, source pages, original/downloaded and final PNG SHA-256 hashes, dimensions, processing, and verified alpha ranges. Refresh with `scripts/fetch_tabletop_assets.py` and `requirements-tabletop-assets.txt`.

Character and sleeve artwork belongs to Bushiroad / Project Vanguard and their credited creators. The template post supplies no explicit open-source license; downloading these assets does not transfer their copyrights.

`glyph-guard.png`, `glyph-target.png`, and `glyph-boost.png` are 64 × 64 white-on-transparent renders of Lucide's Shield, Crosshair, and Zap icons. The ISC and MIT notices are in `lucide-LICENSE.txt`. [Lucide source SVGs](https://github.com/lucide-icons/lucide/tree/56972960bb3cf93fe9208433c21afdf1280cb9eb/icons)

`card_catalog.csv` is a small factual sample of card names, clans, grades, powers, set identifiers, and mechanics for the historical period. It is not a complete card database and contains no card text or images. Sources for card metadata include the [official BT12 list](https://en.cf-vanguard.com/cardlist/cardsearch/?expansion=33&sort=no&view=text), [official BT15 list](https://en.cf-vanguard.com/cardlist/cardsearch/?expansion=42&sort=no&view=text), and the [official Dauntless Drive Dragon entry](https://en.cf-vanguard.com/cardlist/?cardno=BT11%2F005EN&expansion=28&view=image).


## Opening and loading presentation

- `fighters/aichi-standing.png` and `fighters/kai-standing.png` are unchanged transparent `char-stand-01.png` and `char-stand-02.png` from the official anime [Aichi](https://anime.cf-vanguard.com/vg2018-if/characters/aichi-sendou/) and [Kai](https://anime.cf-vanguard.com/vg2018-if/characters/toshiki-kai/) pages. The renderer samples their nonempty bounds; the downloaded files are preserved.
- `presentation/vanguard-logo.png` is the transparent English logo downloaded from the [official English site](https://en.cf-vanguard.com/wordpress/wp-content/themes/cf-vanguard/img/vg_logo_en.png). Source URLs, checksums and alpha verification are included in `tabletop/sources.json` and the existing asset fetcher.
- `presentation/planet-cray.png` is original opaque environment artwork made with the built-in image_gen tool for this project: an anime fantasy canyon, stone causeway and distant citadel beneath a blue sky with moons and a ringed planet. It is inspired by the [official Planet Cray overview](https://en.cf-vanguard.com/introduction/world/) and is not an official background or extracted 3DS asset. The exact prompt and provenance are in `presentation/planet-cray.prompt.txt`.
- `fonts/Rajdhani-SemiBold.ttf` is Rajdhani from the [Google Fonts repository](https://github.com/google/fonts/tree/main/ofl/rajdhani). Its SIL Open Font License is bundled in `fonts/OFL.txt`.
