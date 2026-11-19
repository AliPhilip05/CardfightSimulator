# Official card artwork cache

The PNGs in this directory are resized, palette-optimized local copies of card images served by the official Bushiroad English card database. The artwork, card names, and game marks remain the property of their respective owners. This cache is for the local fan prototype; do not treat it as a redistribution license.

Regenerate the metadata index and image cache from the public card database with:

```sh
python3 -m pip install -r requirements-assets.txt
python3 scripts/fetch_era_card_assets.py
```

The importer covers English card entries listed in BT01–BT15, EB01–EB10, and TD01–TD13. It does not extract or depend on Nintendo 3DS game files. Each metadata record contains a card-list source URL and the original picture path.
