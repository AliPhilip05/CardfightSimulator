# Vanguard Rust Prototype

![Current title screen with Aichi, Kai, and a Planet Cray-inspired landscape](docs/images/title-screen.png)

A small, visual desktop cardfight prototype set in the Limit Break and Break Ride era. The source is split by responsibility so the rules, cards, story, and rendering can be read independently.

## Run

From this project directory:

```sh
cargo run
```

The window opens at 1440 × 960 and can be resized. Startup shows loading progress, followed by an animated title scene with Aichi and Kai over a Planet Cray-inspired landscape. Press Enter or click to open Story Mode, Quick Fight, and the era card library. Entering a match loads decks and opening card art before revealing the field.

## Controls

- **Ride:** click a playable card in your hand. The game highlights legal cards.
- **Call:** click a card in hand, then an empty rear-guard circle. Call as many legal units as you can fit.
- **Attack:** click one of your standing front-row units, then click the opposing Vanguard or an occupied front-row rear-guard.
- **Boost:** when a standing grade 0 or 1 is directly behind the attacker, click it or choose **Boost** to add its power for the battle. Choose **No Boost** to leave it standing.
- **Guard:** for each opponent attack, choose **No Guard** or move one or more highlighted cards to the Guardian Circle. Drag a card from hand onto the circle, or click a hand card or eligible interceptor and then the circle. Choose **Done Guarding** to resolve the attack.
- **Damage Zone / Counter-Blast:** cards lie sideways in one overlapping vertical stack, like the printed mat. During your turn, click a card's exposed strip to turn it face down for manual Counter-Blast; click again to restore it for manual Counter-Charge or to correct a click. `CB` shows available face-up damage. Hover face-up damage for a larger preview. You cannot flip the opponent's damage or change resources during an unresolved attack or after the match ends.
- **End Turn:** passes play to the CPU. Finish guarding or pass on each attack to continue.
- **Limit Break / Break Ride:** activate the corresponding buttons when available. Limit Break automatically pays its own Counter-Blast 2 from available face-up damage; manual flips do not activate an ability or prepay that button's cost.
- **Escape:** return to the main menu.

Story Mode follows a six chapter route through the Card Capital introduction, tournaments and Asia Circuit, VF Koshien, the Reversed fighters, and the Star-vader finale. The Link Joker chapter lets you choose the Hitsue, Fukuhara, or Miyaji school route. The fictionalized story beats and selected cards are based on the 3DS games *Ride to Victory!!* and *Lock on Victory!!*; research notes and sources are in [RESEARCH.md](RESEARCH.md).

## Developer documentation

See the [documentation index](docs/README.md), [architecture and screen flow](docs/architecture.md), and [complete function reference](docs/functions.md). The reference explains every named Rust function and asset-script function, including private helpers and tests.

The [VangPro research report](docs/research/vangpro-research.md) compares public gameplay materials and release-package observations with this codebase, then proposes a staged implementation roadmap.

## Project layout

- `src/main.rs` configures and runs the desktop window.
- `src/ui.rs` owns screen navigation and mouse input.
- `src/ui/board.rs` draws playmat zones, the sideways damage stack, counts, and turn status.
- `src/ui/presentation.rs` draws the shared title/menu/loading scene, official character renders and logo, animated particles, and loading progress.
- `src/ui/cards.rs` draws cards and hit areas; `src/ui/drawing.rs` holds shared vector primitives and palette.
- `src/game/card.rs` defines card, unit, and player data.
- `src/game/deck.rs` builds 50-card clan decks from the bundled official card catalog, with real card names, stats, triggers, and four-copy limits.
- `src/game/engine.rs` implements turn phases, attacks, guards, triggers, and CPU actions.
- `src/story.rs` contains chapter and school-route data.
- `assets/official_era_cards.json` contains the English official card entries used by the library and deck builder. `assets/card_images/` contains 2,083 optimized images sourced from Bushiroad's public card list. The match uses transparent circle artwork in `assets/tabletop/vanguard-zones.png`, Aichi/Kai sleeve PNGs, and the official character icons in `assets/fighters/`. Earlier original and supplied playmats remain in `assets/`.
- `scripts/fetch_era_card_assets.py` refreshes the card data and artwork. Install its optional dependency with `python3 -m pip install -r requirements-assets.txt`, then run `python3 scripts/fetch_era_card_assets.py`.
- `scripts/fetch_tabletop_assets.py` refreshes the playmat, sleeves, icons, and provenance manifest. Install `requirements-tabletop-assets.txt` first. Sleeve SVGs keep downloaded illustrations inside an editable rounded card frame; their PNGs have transparent exterior corners. CairoSVG needs the Cairo system library (on Homebrew macOS, set `DYLD_FALLBACK_LIBRARY_PATH=/opt/homebrew/lib` if needed).

The field follows the classic official guide: three aligned columns, five Rear-guard Circles, a central Vanguard, Guardian Circle toward the opponent, damage on the left, and Trigger/Deck/Drop down the right. The opposing half is mirrored. The damage pocket stacks up to six sideways cards; each has an exposed clickable strip, and hover previews show its full face upright. Hit testing follows the stack's drawing order. The black field, yellow/blue circles, metallic blue rails, and complete green Guardian Circle draw on FuRyu's official Ride to Victory!! screenshots. This prototype retains the Limit Break / Break Ride field rather than adding modern Ride Deck and Order zones.

## Prototype scope

This is a playable foundation, not a complete rules engine. CPU decisions, card effects, guard skills such as sentinels, and story events are simplified. The bundled pool covers English BT01–BT15, EB01–EB10, and TD01–TD13 records available from the official public API; some 3DS-exclusive, Japanese-only, and promotional cards are outside that source. The sample deck builder chooses legal-looking lists from that pool and is not an exact dump of the 3DS opponents' deck recipes. Card names and artwork belong to their respective owners; see [asset provenance](assets/card_images/README.md).
