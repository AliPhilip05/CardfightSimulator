# Application architecture

## Startup and opening sequence

`src/main.rs` configures the resizable Macroquad window and awaits `App::load`. The loader renders real progress between artwork/database/tabletop initialization. `PresentationArt` owns the original landscape, official Aichi/Kai cutouts and logo, and bundled Rajdhani font; the font is shared throughout the app.

```mermaid
flowchart LR
    Title --> Menu
    Menu --> Loading[Build decks and preload art]
    Loading --> RPS[Rock paper scissors]
    RPS -->|Tie| RPS
    RPS --> Order[Winner chooses first or second]
    Order --> Mulligan[One redraw of zero to five cards]
    Mulligan --> Ready[Both hands ready]
    Ready --> StandUp[Stand up Vanguard]
    StandUp --> Stand
    Stand --> Draw
    Draw --> Ride
    Ride --> Main
    Main -->|First global turn| End
    Main -->|Later turn| Battle
    Battle --> End
    End --> Stand
```

Story Mode selects a school route/chapter before loading; Quick Fight uses the first chapter. Library and Result remain separate screens. `App::start_match` clears old selections, log visibility, power animation, and opening-card selections.

`MatchGame::new` prepares a starter Vanguard and five-card hand for each fighter, then waits in Setup. `engine/opening.rs` implements RPS, replayed ties, first/second selection, and a single mulligan. The CPU winner chooses first. Human selections are validated before any card moves. Selected cards go to the bottom (`Vec` index zero), replacements come from the top (`Vec::pop`), then the remaining deck is shuffled. The CPU keeps one card of grades one through three where possible. Both hands stay at five until the first actual Draw phase.

The requested first/second choice and bottom-return mulligan use current setup conventions alongside the historical card pool. They are explicit project behavior; the old 2014 rules used different setup details. No modern Ride Deck or Order zones are added.

## Turn and battle state

`turn` is the global turn number, beginning at one for whichever fighter starts. Human phases include Stand, Draw, Ride, Call/Main, and Battle. CPU phases use `TurnStage` under `Phase::Cpu`. Stand waits 1.5 seconds, including a pause, eased rotation and settling; Draw lasts 0.8 seconds before the single draw commits. Other CPU stages advance every 0.65 seconds. Only the first global turn skips Battle, so the second fighter can attack on their first personal turn.

The field-first HUD uses a floating vertical phase list, enlarged active phase, corner fighter portraits, card-stack/hand icons and counts, compact right-side commands, and an optional log drawer. Routine phase narration is hidden. `MatchPrompt` derives legal decisions without drawing or mutating the match. Opening Main offers End Turn directly. Calls include legal trigger units and overcalls, retiring the previous occupant.

`PendingAttack` advances through Guard → Drive → Damage → Close. The UI waits for human guarding and required trigger choices; other check steps advance every 0.55 seconds. Vanguard swings have one drive check, or two at grade three; rear-guard swings have none. Matching-clan classic triggers grant +5000 power to a chosen unit. Critical and power can target different units; Stand targets a rear-guard; Draw adds one extra card; Heal compares damage before the checked card enters damage and can remove face-down damage. A sixth-damage Heal resolves before checking the six-damage loss condition.

Live battle totals recalculate the attacker, current booster, target and guardians after effects. A damage-trigger defense increase does not cancel the remaining damage checks of an already-hitting critical attack. Temporary power/critical expires at the turn boundary. Locked rear-guards unlock at their owner's End, and stay ineligible during Stand. Empty-deck loss is checked after the current check/effects finish.

The human currently starts check resolution explicitly after the CPU guards. The requested automatic resolution and all-attacks-finished End Turn transition remain pending in [todo.md](../todo.md), with highlighted function links.

Rear-guard slots are shared everywhere:

| Index | Circle |
| --- | --- |
| 0 | Left back |
| 1 | Left front |
| 2 | Center back, behind Vanguard |
| 3 | Right front |
| 4 | Right back |

`AttackUnit` resolves field selections and aligned boosters. Tapping an eligible booster can select its attacker with boosting enabled; tapping it again during target selection toggles the preview. Declaring a boosted attack rests both participants. `GuardSource` unifies hand guardians and eligible front-row grade-two interceptors.

This remains a prototype: CPU decisions, Limit Break effects, Break Ride effects, sentinels, and many printed card abilities are simplified. The engine does not interpret arbitrary card text.

## Geometry, motion, and artwork

`BattlefieldLayout` projects a 720-unit logical field with a slight perspective taper. Rendering and input share circle/card/damage geometry. The arena uses dark green hexagons, yellow rear circles, blue Vanguard circles, a complete green Guardian ellipse, and metallic translucent side-zone frames. Blue column connectors are removed. The era's classic field is mirrored for the opponent.

Damage is one overlapping sideways stack per fighter. Reverse-order hit testing selects the visible top card or exposed strip. Face-up and sleeve views use the same rotated bounds. Manual damage flips update the resource state used by automatic Counter-Blast costs; they do not activate or prepay an ability. Required Heal choices temporarily select damage through the same stack geometry.

`HandLayout` shares overlapping-card drawing and hit testing, with paging for large hands. Draw animation starts at `BattlefieldLayout::deck_rect` and ends at `HandLayout::draw_destination`. Existing cards ease toward the next hand's spacing while the incoming card travels; it flips face up near arrival and commits into the same slot. When the next card belongs to another page, its back shrinks into the paging control and the visible hand stays in place. Opposing draws stay hidden. `motion.rs` provides clamped interpolation and easing. Stand cards rotate smoothly while power/grade anchors follow their rotated bounds. The Stand Up banner fades in and out; the power meter interpolates current totals each frame.

`CardArt` resolves normalized names to bundled printings and loads/caches only needed textures, including the next human card during Draw. `FighterArt` embeds official portraits for Aichi, Kai, Morikawa, Kamui, Misaki, Ren, Asaka, Leon, Naoki, and Takuto. Standing cutouts are also downloaded for future scenes. Unknown fighters use the initial fallback. Newer promotional cosmetics do not change the historical card catalog.

Downloaded artwork and refresh commands are documented in [asset provenance](../assets/README.md). Publisher/game screenshots are research material in [the reference archive](images/references/README.md), separate from runtime art.

## Verification

Run `cargo fmt --check`, `cargo test`, and `cargo build`. Tests cover both starting orders, all RPS pairings, redraw validation and replacement order, one draw per turn, first-turn attack restrictions, paced CPU and human stages, guarding/intercepts, split triggers, sixth-damage Heal, live boosted power, bonus expiry, Lock timing, deck-out, shared field/hand geometry, and artwork resolution. Native review captures verify the phase/portrait/zone layout and opening screens; timing quality also needs play-through review.
