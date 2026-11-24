# Function reference

This reference covers every named function in the Rust application and Python asset scripts, including private helpers and test functions. Paths are relative to the application directory. Methods include their owning type to distinguish names such as `new`, `unit`, and `can_attack`. See [architecture](architecture.md) for how these functions work together.

## `src/main.rs`

Source: [src/main.rs](../src/main.rs).

| Function | Responsibility |
| --- | --- |
| `window_conf` | Returns the window title, initial 1440 × 960 size, resize support, and high-DPI configuration. |
| `main` | Awaits `App::load`, then draws the application and yields to the next Macroquad frame continuously. |

## `src/story.rs`

Source: [src/story.rs](../src/story.rs).

| Function | Responsibility |
| --- | --- |
| `SchoolRoute::name` | Returns the displayed school name for a route. |
| `SchoolRoute::rival` | Returns the route's reversed rival, clan, and boss-card tuple for the Link Joker chapter. |

`CHAPTERS` is data, not a function: its six records supply chapter titles, story summaries, opponents, clans, bosses, Break Ride availability, and rewards.

## `src/game/card.rs`

Source: [src/game/card.rs](../src/game/card.rs).

| Function | Responsibility |
| --- | --- |
| `Card::unit` | Constructs a normal unit with the supplied attributes and no trigger or ability flags. |
| `Card::trigger` | Constructs a grade-zero trigger card with default power/shield values and the requested trigger kind. |
| `Card::is_trigger` | Reports whether the card's trigger kind is other than `None`. |
| `Unit::new` | Wraps a card as a standing, unlocked field unit with no temporary power bonus. |
| `Unit::power` | Returns printed power plus the current temporary bonus. |
| `Player::new` | Creates a player with a deck and empty hand, circles, damage, soul, and drop. |
| `Player::draw` | Pops the deck's top card and updates the deck-out flag according to whether a card was available. |
| `Player::draw_to_hand` | Draws one card and puts it into the hand when available. |
| `Player::grade` | Returns the Vanguard's grade, defaulting to zero when no Vanguard exists. |
| `Player::face_up_damage` | Counts damage cards that are available as face-up resources. |
| `Player::vanguard_power` | Returns the Vanguard's effective power, or zero without a Vanguard. |

## `src/game/deck.rs`

Source: [src/game/deck.rs](../src/game/deck.rs).

| Function | Responsibility |
| --- | --- |
| `build_deck` | Reads the bundled catalog, selects a clan starter, triggers and grade groups, adds boss/Break Ride cards, fills/truncates the sample list to 50, and shuffles it. |
| `add_boss_group` | Adds four copies of the requested boss or a grade-three fallback, plus another grade-three card when available. |
| `add_grade_group` | Adds a requested number of non-trigger units of one grade, taking at most four copies from each selected catalog entry. |
| `find_starter` | Finds a non-trigger grade-zero card in the supplied records. |
| `find_grade` | Finds a non-trigger card of a requested grade, optionally excluding a printing number. |
| `find_named` | Finds a record whose normalized name matches the requested name. |
| `normalize_name` | Removes non-alphanumeric characters and lowercases a name for deck lookup. |
| `find_break_ride` | Looks up the prototype's named Break Ride unit for Kagero, Narukami, or Link Joker. |
| `unique_units` | Collects non-trigger catalog entries of a requested grade; despite the helper's name, it does not perform a separate name-deduplication pass. |
| `grade` | Parses a catalog record's grade string into an optional integer. |
| `card_number` | Returns a catalog printing number, or an empty string if absent. |
| `is_trigger` | Recognizes Critical, Draw, Stand, or Heal in a catalog trigger description. |
| `trigger_kind` | Converts the catalog trigger description to a `Trigger` enum value. |
| `to_unit` | Converts a JSON record into a normal `Card`, parsing factual attributes and applying fallback values where needed. |
| `to_trigger` | Converts a JSON record into a trigger `Card` while preserving available printed power and shield. |
| `leaked` | Allocates a catalog string with static lifetime for the prototype's `Card` representation. |
| `prepare_player` | Creates a player, removes a grade-zero starter from the deck into the Vanguard Circle, and draws five opening hand cards. |
| `shuffle` | Uses a time-seeded xorshift/Fisher–Yates-style pass to reorder a mutable slice. |

## `src/game/engine.rs`

Source: [src/game/engine.rs](../src/game/engine.rs).

| Function | Responsibility |
| --- | --- |
| `AttackUnit::boost_slot` | Maps an attacking front-row unit to its directly aligned back-row booster; other rear slots have none. |
| `AttackUnit::unit` | Resolves a Vanguard/rear selection to an immutable unit reference. |
| `AttackUnit::unit_mut` | Resolves that selection to a mutable unit reference. |
| `AttackUnit::can_attack` | Checks that the selected unit is a valid unlocked front-row unit and is standing. |
| `AttackUnit::can_be_targeted` | Checks that the selection identifies an occupied, unlocked front-row circle. |
| `AttackUnit::booster` | Returns a standing, unlocked grade-zero/one unit in the aligned booster slot. |
| `PendingAttack::shield` | Sums the shield values of the battle's guardians. |
| `MatchGame::new` | Builds both chapter decks, prepares their players, initializes the first Ride phase, and records the opening log message. |
| `MatchGame::log` | Appends a log entry while retaining only the most recent eight messages. |
| `MatchGame::ride_options` | Returns hand indices of non-trigger cards at the current or next Vanguard grade, up to grade three. |
| `MatchGame::call_options` | Returns hand indices of non-trigger units no higher than the Vanguard's grade. |
| `MatchGame::ride` | Moves the selected hand card into Vanguard, places the previous Vanguard in soul, applies the prototype's Break Ride bonus/restand flag, logs the action, and enters Main/Call phase. |
| `MatchGame::skip_ride` | Advances from Ride to Main/Call without replacing the Vanguard. |
| `MatchGame::call` | Places a playable hand unit into an empty rear-guard slot during Main/Call and updates the call counter and log. |
| `MatchGame::begin_battle` | Advances from Main/Call to Battle. |
| `MatchGame::can_limit_break` | Requires Main/Battle phase, no pending attack, a flagged Vanguard, at least four damage, and two face-up damage. |
| `MatchGame::can_flip_damage` | Allows manual damage changes only during the player's active phases, without a pending battle or match result. |
| `MatchGame::flip_damage` | Toggles one valid player damage card's face-up state, logs Counter-Blast or manual restoration, and returns whether a change occurred. |
| `MatchGame::use_limit_break` | Pays Counter-Blast 2 and applies the prototype's selected Vanguard effect, including named lock/retire variants or a power bonus. |
| `MatchGame::can_attack` | Combines the player's Battle phase and no-pending-battle requirements with the selected unit's attack eligibility. |
| `MatchGame::attack_targets` | Lists occupied, unlocked opposing Vanguard/front rear-guard targets. |
| `MatchGame::boost_for` | Returns the selected attacker's eligible aligned booster. |
| `MatchGame::declare_attack` | Validates a player attack and opens its pending battle with the requested boost choice. |
| `MatchGame::can_guard` | Checks whether a selected hand card can guard the pending CPU attack. |
| `MatchGame::call_guardian` | Moves a valid selected hand card to the pending battle's Guardian Circle. |
| `MatchGame::can_intercept` | Checks whether a selected rear-guard can intercept the pending CPU attack. |
| `MatchGame::intercept` | Removes an eligible rear-guard from its circle and adds it as a guardian. |
| `MatchGame::player_guard_shield` | Resolves a guard source's legal shield only when the pending attack belongs to the CPU. |
| `MatchGame::add_player_guardian` | Validates and consumes a hand/intercept source, adds its card to guardians, logs the move, and returns success. |
| `MatchGame::resolve_pending_attack` | Consumes the pending battle, resolves its outcome, moves guardians to drop, records the result, and checks match completion. |
| `MatchGame::can_restand` | Checks for the player's pending Dauntless-style Break Ride restand, a rested Vanguard, sufficient hand cost, and no unresolved attack. |
| `MatchGame::use_restand` | Pays the three-card discard, stands the Vanguard, clears the one-use restand flag, and logs the action. |
| `MatchGame::end_turn` | Starts the CPU turn, readies/draws for it, checks deck-out, performs its ride/calls/ability, and starts its next attack. |
| `MatchGame::advance_cpu_turn` | Opens CPU attacks one at a time, handles its simplified extra restand attack, then unlocks units and starts the player's next turn. |
| `MatchGame::declare_cpu_attack` | Chooses a CPU attack target based on power and available opposing front units, then opens a boosted/unboosted pending battle. |
| `MatchGame::open_attack` | Validates attacker/target/booster, rests attacking participants, computes initial battle power, chooses CPU guardians when appropriate, and creates `PendingAttack`. |
| `MatchGame::start_player_turn` | Increments the turn, readies and draws for the player, resets per-turn counters/flags, enters Ride, and checks defeat conditions. |
| `MatchGame::capture_outcome` | Ends the fight when either player has six damage or has failed a deck draw. |
| `pay_counter_blast` | Turns the requested number of available face-up damage cards face down, scanning in zone order. |
| `ready_player` | Stands field units and clears temporary power bonuses at the start of their turn. |
| `unlock_rear_guards` | Clears locked state from all rear-guards. |
| `cpu_ride` | Chooses the highest eligible ride in the CPU hand and moves its previous Vanguard into soul. |
| `cpu_call` | Calls a high-power playable unit to the first empty rear-guard slot. |
| `cpu_limit_break` | Pays the CPU's simplified Counter-Blast cost, grants its Vanguard power, and performs the selected Link Joker lock behavior. |
| `restand_vanguard` | Discards the final three hand cards to drop and stands the Vanguard; callers must validate the cost first. |
| `resolve_attack` | Performs drive checks for a Vanguard attacker, compares current attack power to defense plus shield, and returns miss, retired rear-guard, or dealt damage. |
| `drive_check` | Reveals one/two deck cards, applies matching-clan triggers, records the latest check, adds checked cards to hand, and returns battle critical. |
| `GuardSource::shield` | Computes legal positive shield for a hand guardian or eligible grade-two front-row interceptor that is not the attack target. |
| `GuardSource::take_card` | Removes a validated hand/interceptor source and returns its card. |
| `auto_guard` | Selects enough eligible low-shield guardians/interceptors to defend, or passes if their total shield cannot meet the requirement. |
| `deal_damage` | Performs damage checks, applies matching-clan triggers, appends face-up damage cards, stops at six/deck exhaustion, and returns the number dealt. |
| `matches_clan` | Checks whether a card matches the clan of the player's Vanguard or any rear-guard. |
| `apply_trigger` | Grants temporary Vanguard power and applies the prototype's Stand, Draw, or eligible Heal effect. Critical is counted by the drive-check caller. |

### Engine test helpers and tests

| Function | What it establishes |
| --- | --- |
| `tests::card` | Creates a small Royal Paladin test card with specified grade, power, and shield. |
| `tests::unit` | Wraps a test card as an optional field unit. |
| `tests::battle` | Constructs a controlled Battle-phase fixture with predictable nonmatching-clan deck cards. |
| `tests::flipping_damage_updates_costs_without_moving_or_reordering_cards` | Checks manual flips, automatic cost availability, invalid indices, and preservation of the damage zone. |
| `tests::damage_cannot_be_manually_changed_during_opponent_turn_battle_or_result` | Rejects resource edits during pending attacks, CPU/finished phases, and completed results. |
| `tests::attack_targets_only_include_occupied_unlocked_front_row_units` | Rejects locked/back-row targets and verifies target eligibility. |
| `tests::each_column_uses_its_own_optional_booster` | Verifies alignment, optional boosting, and resting of the correct participants. |
| `tests::each_cpu_attack_pauses_for_a_guard_decision` | Verifies individual CPU battle pauses, hand guardians, and resolution flow. |
| `tests::rested_front_rear_guard_can_intercept_for_another_unit` | Allows a rested eligible interceptor while preserving target restrictions. |
| `tests::drive_check_can_punch_through_a_guard_for_a_rear_guard_target` | Verifies that a drive-trigger power change can alter a guarded battle against a rear-guard. |
| `tests::stand_allows_another_rear_guard_attack_but_critical_does_not` | Distinguishes standing another attacker from adding critical to the current battle. |

## `src/ui.rs`

Source: [src/ui.rs](../src/ui.rs).

| Function | Responsibility |
| --- | --- |
| `App::load` | Loads opening artwork/font, renders database/tabletop initialization progress across frames, creates UI state and the card-art resolver, and enters Title. |
| `App::draw` | Clears the frame, draws applicable ambient effects, and dispatches the active screen; Title input opens Menu. |
| `App::draw_menu` | Draws the shared opening scene and Story/Quick Fight/Library buttons; Escape returns to Title. |
| `App::draw_match_loading` | Renders stage progress, constructs the chapter fight, preloads visible opening art, handles cancellation, and enters Match after completion. |
| `App::draw_routes` | Draws school choices, applies a selected story route, and starts loading its first chapter. |
| `App::draw_match` | Advances CPU timing, loads visible art, builds layouts, draws the field/hand/controls/log, and dispatches actions or mouse input. |
| `App::battlefield_highlights` | Derives selectable attackers, targets, boosters, empty call circles, and guardians from current phase and selections. |
| `App::draw_controls` | Draws the contextual instruction strip and legal command buttons, returning an optional `MatchAction`. |
| `App::apply_match_action` | Clears transient selections and maps a command to the appropriate engine method, timer, or result screen. |
| `App::handle_match_clicks` | Handles Escape, damage flips, hand selection, rides, calls, attacker/target/booster clicks, and guardian click/drag/drop input. |
| `App::is_dragging_guard` | Compares the cursor to its recorded guard press and reports whether it moved past the drag threshold. |
| `App::draw_result` | Presents win/loss, chapter summary/reward, replay/next chapter behavior, and a route back to Menu. |
| `App::draw_library` | Handles search text, filtering and paging, loads visible catalog art, draws results, and opens/closes card details. |
| `App::start_match` | Clears old fight/selections, resets loading state/timers, and enters Loading; deck creation happens in the loading stage. |
| `embedded_texture` | Decodes a bundled PNG into a linearly filtered Macroquad texture. |
| `field_unit_at` | Resolves a screen point to the Vanguard or a rear-guard selection on one side. |
| `mark_field_unit` | Writes a color highlight into the chosen fighter/circle entry. |
| `unit_name` | Returns a selected unit's card name, with a generic fallback. |
| `visible_card_names` | Collects hand, field, face-up damage, guardian, latest-check, and top-drop names needed for the current texture preload. |
| `checked_card_name` | Removes the `Drive: ` or `Damage: ` prefix from a recorded check before card-art lookup. |
| `draw_card_detail` | Draws the selected catalog card's image and factual/printed detail text in the library overlay. |

## `src/ui/presentation.rs`

Source: [src/ui/presentation.rs](../src/ui/presentation.rs).

| Function | Responsibility |
| --- | --- |
| `PresentationArt::new` | Decodes the embedded landscape, Aichi/Kai cutouts, logo, and title font. |
| `PresentationArt::scale` | Computes a bounded presentation scale from the current window dimensions. |
| `PresentationArt::text` | Draws text using the shared Rajdhani font, requested color, and a minimum readable size. |
| `PresentationArt::centered` | Measures Rajdhani text and positions it around a requested center. |
| `PresentationArt::scene` | Covers the window with the landscape, draws smooth readability gradients, positions/bobs both characters, animates particles, and draws the footer. |
| `PresentationArt::character` | Draws an unchanged character texture using its nonempty source bounds and proportional destination height. |
| `PresentationArt::heading` | Draws the official logo with a shadow and the prototype's era/title captions. |
| `PresentationArt::title` | Draws the title scene and pulsing start prompt; returns whether Enter or a mouse press requests Menu. |
| `PresentationArt::menu_button` | Draws a responsive menu row with title/detail/hover state and reports a click. |
| `PresentationArt::loading` | Draws the shared scene with completed-stage progress, status text, and an animated spinner. |
| `gradient` | Draws one interpolated four-corner mesh so translucent scene gradients have no strip seams. |

## `src/ui/art.rs`

Source: [src/ui/art.rs](../src/ui/art.rs).

| Function | Responsibility |
| --- | --- |
| `CardArt::new` | Builds a resolver using the current executable, compiled project root, current directory, and catalog records. |
| `CardArt::from_layout` | Builds normalized-name-to-printing mappings and candidate roots from explicitly supplied paths. |
| `CardArt::preload` | Loads missing requested textures, remembers unavailable names, and evicts unrelated cached textures when approaching the cache limit. |
| `CardArt::textures` | Returns a read-only reference to the loaded texture map. |
| `CardArt::load_image` | Tries every matching printing against valid asset roots until one PNG decodes successfully. |
| `artwork_roots` | Finds distinct valid install/app-resource/Cargo-project/working-directory roots containing card assets. |
| `normalize_card_name` | Removes non-alphanumeric characters and lowercases card names for artwork lookup. |
| `tests::local_card_art_decodes_with_an_unrelated_working_directory` | Confirms that bundled card art can load through the project root without relying on the launch directory. |
| `tests::missing_printing_can_fall_back_to_another_printing` | Confirms that a missing printing can fall back to another path for the same normalized name. |

## `src/ui/board.rs`

Source: [src/ui/board.rs](../src/ui/board.rs).

| Function | Responsibility |
| --- | --- |
| `TabletopArt::sleeve` | Chooses the player's Aichi sleeve or opponent's Kai sleeve. |
| `BattlefieldLayout::new` | Fits a proportion-preserving logical tabletop into available screen bounds. |
| `BattlefieldLayout::point` | Converts a logical tabletop point to screen coordinates. |
| `BattlefieldLayout::rect` | Converts a logical center/size into a scaled screen rectangle. |
| `BattlefieldLayout::side_point` | Mirrors a player's local point across the tabletop for the opposing fighter. |
| `BattlefieldLayout::row_center` | Returns the logical center of an aligned front/back column for one fighter. |
| `BattlefieldLayout::rear_center` | Maps one of the five fixed rear-guard indices to its circle center. |
| `BattlefieldLayout::vanguard_center` | Returns the middle front-row circle center. |
| `BattlefieldLayout::guardian_center` | Returns the shared central Guardian Circle center. |
| `BattlefieldLayout::damage_center` | Returns a fighter's damage-pocket center on the left of their field. |
| `BattlefieldLayout::damage_card` | Returns the sideways screen rectangle for an index in the vertically overlapping damage stack. |
| `BattlefieldLayout::damage_at` | Searches occupied damage cards from top to bottom in draw order, making both overlaps and exposed strips select the visible card. |
| `BattlefieldLayout::unit_at` | Tests a screen point against a circle and its card rectangle. |
| `BattlefieldLayout::rear_at` | Returns the rear-guard slot hit by a point. |
| `BattlefieldLayout::vanguard_at` | Reports whether a point hits the Vanguard circle/card. |
| `BattlefieldLayout::guardian_at` | Tests both halves of the full Guardian ellipse. |
| `BattlefieldLayout::label` | Draws a centered, scaled label at a logical position. |
| `draw_battlefield` | Composes the surface, side zones, damage stacks, circles, field units, soul counts, guardians, and hovered damage preview. |
| `draw_table_surface` | Draws the black arena, subtle grid, metallic blue rails, column connectors, and center divider. |
| `draw_mat_art` | Samples and layers a transparent circle graphic at a field position with the appropriate tint/orientation. |
| `draw_circle_unit` | Draws a circle's interactive highlight and optional unit card. |
| `draw_guardians` | Draws a complete green Guardian ring and hit highlight, then distributes the pending battle's guardian cards across it. |
| `draw_damage` | Draws one printed pocket, sideways overlapping face/sleeve cards, hover selection, total damage, and face-up CB resources; returns a hovered face-up card for inspection. |
| `draw_damage_preview` | Places a full upright damage-card face and caption beside its stack, clamped to field bounds. |
| `draw_sleeve` | Draws a proportionally supplied card-back rectangle, rotating the opponent's normal deck/check view. Sideways damage backs use their own quarter-turn in `draw_damage`. |
| `draw_side_zones` | Draws Trigger/Deck/Drop pockets, deck sleeves/counts, latest checked artwork, and the top drop card. |
| `draw_count` | Draws a compact dark badge and centered count text. |
| `tint` | Returns an existing color with a replacement alpha value. |
| `tests::each_front_circle_aligns_with_its_back_circle` | Verifies matching column coordinates and mirrored row positions. |
| `tests::shrinking_the_table_preserves_unit_and_guardian_hit_areas` | Verifies scaled selection geometry on a smaller field. |
| `tests::overlapping_damage_stays_in_one_pocket_and_each_exposed_strip_is_clickable` | Checks one-to-six sideways overlapping cards, exposed-strip/topmost selection, opposite-side isolation, and containment at several sizes. |
| `tests::guardian_accepts_cards_on_both_halves_of_the_complete_circle` | Checks valid points on the upper and lower Guardian halves and rejects a point outside them. |

## `src/ui/cards.rs`

Source: [src/ui/cards.rs](../src/ui/cards.rs).

| Function | Responsibility |
| --- | --- |
| `draw_field_unit` | Draws a standing/rested field card, lock overlay when needed, and effective-power tag. |
| `draw_guard_card` | Draws a guardian card sideways using the common face renderer. |
| `draw_card_face` | Draws full card art or a clan/name fallback, preserves quarter-turn geometry when requested, and adds a clan-colored edge. |
| `draw_clan_art` | Draws a small vector emblem used when a card face is unavailable. |
| `HandLayout::new` | Calculates readable hand-card size, overlap spacing, page capacity, visible count, and a clamped page. |
| `HandLayout::visible_indices` | Returns the hand-index range on the active page. |
| `HandLayout::card_rect` | Returns a visible hand card's rectangle. |
| `HandLayout::exposed_width` | Returns how much of a hand card is visible before the next overlapping card. |
| `HandLayout::card_at` | Finds the visible hand card under a point, checking overlap in topmost order. |
| `HandLayout::draw_page_controls` | Draws previous/next controls when required and returns a requested page change. |
| `draw_player_hand` | Draws the hand ribbon, playable/guard/selected highlights, captions and shield labels, plus an enlarged hovered card. |
| `hand_card_caption` | Formats the card name, grade, power and shield summary used by the hand. |
| `draw_shield_label` | Fits a positive shield indicator within the card's exposed hand segment. |
| `draw_hand_preview` | Draws a larger upright card preview near the hand panel. |
| `fit_caption` | Fits a label to available width, truncating with an ellipsis when necessary. |
| `tests::overlapping_hand_cards_use_the_visible_card_for_input` | Verifies that overlapping hand rectangles choose the exposed/topmost card. |
| `tests::every_hand_card_remains_reachable_when_the_ribbon_pages` | Verifies access to all hand cards across page ranges. |

## `src/ui/match_hud.rs`

Source: [src/ui/match_hud.rs](../src/ui/match_hud.rs).

| Function | Responsibility |
| --- | --- |
| `MatchLayout::new` | Allocates responsive header, board, hand, command-strip, and optional log rectangles. |
| `draw_match_shell` | Draws the desktop match chrome and panel separators behind game content. |
| `draw_match_hud` | Draws fighter portraits/status, turn and phase indicators, and the log toggle; returns a toggle click. |
| `fighter_counts` | Formats damage, deck, and hand counts for one fighter. |
| `draw_battle_log` | Draws a sized overlay of recent log entries, emphasizing the latest entry. |
| `draw_command_button` | Draws a command button's primary/secondary/hover appearance and reports a click. |
| `draw_right` | Positions a measured text label against a right edge. |
| `inset` | Returns a rectangle reduced on every side by a given amount. |
| `rounded_box` | Draws a rounded rectangle using rectangles and corner circles, guarding against invalid sizes. |

## `src/ui/drawing.rs`

Source: [src/ui/drawing.rs](../src/ui/drawing.rs).

| Function | Responsibility |
| --- | --- |
| `draw_ambient_background` | Draws the lightweight abstract background used on secondary screens. |
| `draw_header` | Draws a secondary-screen title/subtitle bar with an accent rule. |
| `button` | Draws a standard rectangular button with hover state and reports a left click. |
| `draw_round_panel` | Draws the prototype's framed panel and offset shadow. |
| `draw_centered` | Measures and horizontally centers text in a supplied width. |
| `draw_wrapped` | Word-wraps text within a width and maximum line count. |
| `draw_star_emblem` | Draws the larger circular star motif used on non-title screens. |
| `draw_star` | Draws a star-shaped vector emblem at the requested position/radius/color. |
| `draw_school_crest` | Draws a route/school emblem using one of the selected vector styles. |
| `clan_color` | Returns a consistent accent color for recognized clans or the fallback clan palette. |

## `scripts/fetch_era_card_assets.py`

Source: [scripts/fetch_era_card_assets.py](../scripts/fetch_era_card_assets.py).

| Function | Responsibility |
| --- | --- |
| `Pacer.__init__` | Initializes the shared request interval, lock and next-request time. |
| `Pacer.wait` | Spaces network requests across download workers using a lock and monotonic time. |
| `read_url` | Downloads bytes with source-identifying headers, pacing, bounded retries, and a final descriptive error. |
| `fetch_json` | Downloads and parses an API JSON response. |
| `included_product` | Recognizes products in the BT01–BT15, EB01–EB10, and TD01–TD13 scope. |
| `fetch_products` | Fetches, filters, and sorts the official product records. |
| `fetch_expansion_cards` | Fetches all paginated card entries for an expansion. |
| `image_filename` | Produces a filesystem-safe PNG filename from a card number. |
| `image_url` | Preserves an absolute image URL or resolves a picture path against the official image base. |
| `download_and_optimize` | Reuses an existing image or downloads it, bounds its dimensions, flattens transparency, quantizes to PNG, and returns its local path. |
| `write_metadata` | Writes the bundled factual card JSON and CSV source index from products and cards. |
| `main` | Collects/deduplicates era records, writes metadata, downloads artwork with bounded worker concurrency, reports progress/failures, and returns an exit status. |

## `scripts/fetch_tabletop_assets.py`

Source: [scripts/fetch_tabletop_assets.py](../scripts/fetch_tabletop_assets.py).

| Function | Responsibility |
| --- | --- |
| `main` | Downloads the character/icon/logo/sleeve/template sources, preserves direct PNGs, packages sleeve JPEGs in rounded SVG frames, exports the existing PSD composite to PNG, verifies transparency, and writes source/output checksums to the provenance manifest. It does not regenerate the landscape or download the font. |

`src/game/mod.rs` only declares modules and contains no functions. JSON/CSV catalogs, SVGs, images, fonts, and requirement files are data/assets rather than executable functions.
