# Vanguard Rust Prototype

A small, visual desktop cardfight prototype set in the Limit Break and Break Ride era. The source is split by responsibility so the rules, cards, story, and rendering can be read independently.

## Run

From this project directory:

```sh
cargo run
```

The window opens at 1440 × 960 and can be resized. The opening screen offers Story Mode, Quick Fight, and a searchable era card library with official card art.

## Controls

- **Ride:** click a playable card in your hand. The game highlights legal cards.
- **Call:** click a card in hand, then an empty rear-guard circle. Call as many legal units as you can fit.
- **Attack:** click one of your standing front-row units, then click the opposing Vanguard or an occupied front-row rear-guard.
- **Boost:** when a standing grade 0 or 1 is directly behind the attacker, click it or choose **Boost** to add its power for the battle. Choose **No Boost** to leave it standing.
- **Guard:** for each opponent attack, choose **No Guard** or move one or more highlighted cards to the Guardian Circle. Drag a card from hand onto the circle, or click a hand card or eligible interceptor and then the circle. Choose **Done Guarding** to resolve the attack.
- **Damage Zone:** damage cards appear in the printed stack on each playmat, with card backs for face-down damage.
- **End Turn:** passes play to the CPU. Finish guarding or pass on each attack to continue.
- **Limit Break / Break Ride:** activate the corresponding buttons when available.
- **Escape:** return to the main menu.

Story Mode follows a six chapter route through the Card Capital introduction, tournaments and Asia Circuit, VF Koshien, the Reversed fighters, and the Star-vader finale. The Link Joker chapter lets you choose the Hitsue, Fukuhara, or Miyaji school route. The fictionalized story beats and selected cards are based on the 3DS games *Ride to Victory!!* and *Lock on Victory!!*; research notes and sources are in [RESEARCH.md](RESEARCH.md).

## Project layout

- `src/main.rs` configures and runs the desktop window.
- `src/ui.rs` owns screen navigation and mouse input.
- `src/ui/board.rs` draws playmat zones, counts, and turn status.
- `src/ui/cards.rs` draws cards and hit areas; `src/ui/drawing.rs` holds shared vector primitives and palette.
- `src/game/card.rs` defines card, unit, and player data.
- `src/game/deck.rs` builds 50-card clan decks from the bundled official card catalog, with real card names, stats, triggers, and four-copy limits.
- `src/game/engine.rs` implements turn phases, attacks, guards, triggers, and CPU actions.
- `src/story.rs` contains chapter and school-route data.
- `assets/official_era_cards.json` contains the English official card entries used by the library and deck builder. `assets/card_images/` contains 2,083 optimized images sourced from Bushiroad's public card list. `assets/playmat.svg` and `assets/playmat.png` are earlier original artwork. The match uses the supplied `assets/supplied-playmat.png` for both fighters.
- `scripts/fetch_era_card_assets.py` refreshes the card data and artwork. Install its optional dependency with `python3 -m pip install -r requirements-assets.txt`, then run `python3 scripts/fetch_era_card_assets.py`.

## Prototype scope

This is a playable foundation, not a complete rules engine. CPU decisions, card effects, guard skills such as sentinels, and story events are simplified. The bundled pool covers English BT01–BT15, EB01–EB10, and TD01–TD13 records available from the official public API; some 3DS-exclusive, Japanese-only, and promotional cards are outside that source. The sample deck builder chooses legal-looking lists from that pool and is not an exact dump of the 3DS opponents' deck recipes. Card names and artwork belong to their respective owners; see [asset provenance](assets/card_images/README.md).
