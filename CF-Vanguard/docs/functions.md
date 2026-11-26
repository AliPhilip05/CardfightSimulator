# Function reference

This reference covers every named Rust function and Python asset-script function, including private helpers and tests. Paths are relative to the application directory. See [architecture](architecture.md) for the flow and [pending gameplay work](../todo.md) for the highlighted trigger/pause/end-turn functions.

## `src/main.rs`

Source: [src/main.rs](../src/main.rs).

| Function | Responsibility |
| --- | --- |
| `application_icon` | Returns embedded 16/32/64 RGBA icon data for direct launches; packaged macOS apps keep the multi-resolution ICNS in Resources. |
| `window_conf` | Returns the window title, initial 1440 × 960 size, resize support, high-DPI configuration, and the custom icon. |
| `main` | Awaits `App::load`, then draws the application and yields to the next Macroquad frame continuously. |

## `src/story.rs`

Source: [src/story.rs](../src/story.rs).

| Function | Responsibility |
| --- | --- |
| `SchoolRoute::name` | Returns the displayed school name for a route. |
| `SchoolRoute::rival` | Returns the route's reversed rival, clan, and boss-card tuple for the Link Joker chapter. |

## `src/game/card.rs`

Source: [src/game/card.rs](../src/game/card.rs).

| Function | Responsibility |
| --- | --- |
| `Card::unit` | Constructs a normal unit with the supplied attributes and no trigger or ability flags. |
| `Card::trigger` | Creates a grade-zero trigger with default 5000 power; Draw has 5000 shield and the other classic triggers have 10000. |
| `Card::is_trigger` | Reports whether the card's trigger kind is other than `None`. |
| `Unit::new` | Creates a standing, unlocked unit with no power or critical bonuses. |
| `Unit::critical` | Returns base critical one plus the current turn’s critical bonus. |
| `Unit::power` | Returns printed power plus the current temporary bonus. |
| `Player::new` | Creates a player with a deck and empty hand, circles, damage, soul, and drop. |
| `Player::draw` | Pops one deck card and marks deck-out when the deck becomes empty, including after its final successful draw. |
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
| `normalize_name` | Removes non-alphanumeric characters and lowercases a name for deck lookup. |
| `grade` | Parses a catalog record's grade string into an optional integer. |
| `card_number` | Returns a catalog printing number, or an empty string if absent. |
| `is_trigger` | Recognizes Critical, Draw, Stand, or Heal in a catalog trigger description. |
| `trigger_kind` | Converts the catalog trigger description to a `Trigger` enum value. |
| `to_unit` | Converts a JSON record into a normal `Card`, parsing factual attributes and applying fallback values where needed. |
| `to_trigger` | Converts a JSON record into a trigger `Card` while preserving available printed power and shield. |
| `leaked` | Allocates a catalog string with static lifetime for the prototype's `Card` representation. |
| `card_from_record` | Converts an editor catalog record into a playable card with printed stats, trigger type, and supported Limit Break/Break Ride flags. |
| `prepare_player` | Creates a player, removes a grade-zero starter from the deck into the Vanguard Circle, and draws five opening hand cards. |
| `find_starter` | Finds a non-trigger grade-zero card in the supplied records. |
| `find_grade` | Finds a non-trigger card of a requested grade, optionally excluding a printing number. |
| `find_named` | Finds a record whose normalized name matches the requested name. |
| `find_break_ride` | Looks up the prototype's named Break Ride unit for Kagero, Narukami, or Link Joker. |
| `unique_units` | Collects non-trigger catalog entries of a requested grade; despite the helper's name, it does not perform a separate name-deduplication pass. |
| `shuffle` | Uses a time-seeded xorshift/Fisher–Yates-style pass to reorder a mutable slice. |

## `src/game/deck_editor.rs`

Source: [src/game/deck_editor.rs](../src/game/deck_editor.rs).

| Function | Responsibility |
| --- | --- |
| `CardSort::label` | Labels the five top-toolbar sort choices. |
| `CatalogFilter::results` | Intersects nation, clan, grade-mask, and search filters; sorts matching indices numerically where appropriate with deterministic name/number ties. |
| `field` | Reads a string catalog field with an empty fallback. |
| `nation` | Returns the source nation, correcting misplaced Spike Brothers printings to Dark Zone. |
| `number` | Parses a numeric catalog field with a zero fallback. |
| `hierarchy` | Builds the default nation-index, clan-name, numeric-grade ordering key. |
| `is_trigger` | Recognizes classic Critical, Draw, Stand, and Heal metadata. |
| `EditedDeck::copies` | Counts copies by card name across printings. |
| `EditedDeck::add` | Adds a printing after checking its index, 50-card capacity, shared four-copy maximum, and clan. |
| `EditedDeck::remove` | Removes the last copy of an exact printing. |
| `EditedDeck::validation` | Checks catalog indices, 50 cards, one clan, named-copy limits, 16 triggers, no more than four Heals, and a normal Grade 0 starter. |
| `EditedDeck::playable` | Validates, converts selected records into engine cards, and shuffles the list. |
| `EditedDeck::sample` | Builds a bounded-copy clan recipe with a starter, sixteen triggers, and 14/11/8 units in grades 1/2/3; missing trigger types use other names. |
| `EditedDeck::save` | Validates and atomically writes printing-number JSON via a temporary file in the platform data folder. |
| `EditedDeck::load` | Reads the saved file, treats an absent file as no saved deck, and validates its resolved list. |
| `EditedDeck::from_saved_json` | Parses printing-number JSON, resolves every number to its catalog entry, and rejects unavailable or invalid lists. |
| `saved_deck_path` | Selects macOS Application Support, Windows APPDATA, or Linux XDG/local-share storage. |
| `tests::catalog` | Reads the bundled catalog for editor tests. |
| `tests::filters_intersect_and_grades_combine` | Checks unfiltered results, intersecting clan/nation/search, combined grades, and empty matches. |
| `tests::sorts_are_numeric_and_default_preserves_hierarchy` | Checks complete deterministic results and ordering for all five sorts. |
| `tests::reprints_share_copy_limit_and_mixed_clans_are_rejected` | Checks shared-name copy limits, mixed-clan rejection, and exact-printing removal. |
| `tests::nation_filter_corrects_spike_brothers_source_errors` | Checks all 44 Spike Brothers printings appear under Dark Zone and none under Zoo. |
| `tests::saved_decks_round_trip_and_reject_missing_or_invalid_cards` | Checks saved-list round trips and malformed/missing-card rejection, then starts a custom Kagero opening with five hand cards and 44 deck cards. |
| `tests::samples_validate_and_playable_decks_keep_card_stats` | Validates samples for all 23 clans, their 50 playable cards and sixteen triggers, and rejects an incomplete list. |

## `src/game/engine.rs`

Source: [src/game/engine.rs](../src/game/engine.rs).

| Function | Responsibility |
| --- | --- |
| `TurnStage::label` | Returns the visible uppercase name of a Stand, Draw, Ride, Main, Battle, or End stage. |
| `AttackUnit::boost_slot` | Maps an attacking front-row unit to its directly aligned back-row booster; other rear slots have none. |
| `AttackUnit::unit` | Resolves a Vanguard/rear selection to an immutable unit reference. |
| `AttackUnit::unit_mut` | Resolves that selection to a mutable unit reference. |
| `AttackUnit::can_attack` | Checks that the selected unit is a valid unlocked front-row unit and is standing. |
| `AttackUnit::can_be_targeted` | Checks that the selection identifies an occupied, unlocked front-row circle. |
| `AttackUnit::booster` | Returns a standing, unlocked grade-zero/one unit in the aligned booster slot. |
| `TriggerPart::instruction` | Returns the short instruction for a power, critical, Stand, or Heal choice. |
| `CheckReveal::new` | Captures a numbered drive/damage draw, checking side, real card, clan eligibility, and Heal/Stand availability before effects resolve. |
| `PendingAttack::checking_cpu` | Identifies the checking fighter: the attacker for drive checks, the defender for damage checks. |
| `PendingAttack::shield` | Sums the shield values of the battle's guardians. |
| `MatchGame::new` | Builds the two sample decks and five-card hands, then enters rock-paper-scissors setup without taking a turn draw. |
| `MatchGame::with_player_deck` | Prepares a supplied validated player deck and the chapter CPU deck, then enters the normal RPS/opening sequence. |
| `MatchGame::log` | Appends a log entry while retaining only the most recent eight messages. |
| `MatchGame::stage` | Returns the current Stand/Draw/Ride/Main/Battle/End stage, or none during setup/results. |
| `MatchGame::ride_options` | Lists same-grade or next-grade hand cards, including triggers when legal, up to grade three. |
| `MatchGame::call_options` | Lists hand units no higher than the Vanguard’s grade, including trigger units. |
| `MatchGame::ride` | Validates before consuming the hand card, moves the previous Vanguard into soul, applies the simplified Break Ride effect, and enters Main. |
| `MatchGame::skip_ride` | Advances from Ride to Main/Call without replacing the Vanguard. |
| `MatchGame::call` | Calls a legal hand unit to a rear circle, retiring an existing occupant to drop when overcalling. |
| `MatchGame::begin_battle` | Enters Battle from Main only after the starting fighter’s first turn. |
| `MatchGame::can_limit_break` | Requires Main/Battle phase, no pending attack, a flagged Vanguard, at least four damage, and two face-up damage. |
| `MatchGame::can_flip_damage` | Allows manual damage changes only during the player's active phases, without a pending battle or match result. |
| `MatchGame::flip_damage` | Toggles one valid player damage card's face-up state, logs Counter-Blast or manual restoration, and returns whether a change occurred. |
| `MatchGame::use_limit_break` | Pays Counter-Blast 2 and applies the prototype's selected Vanguard effect, including named lock/retire variants or a power bonus. |
| `MatchGame::can_attack` | Requires a later human Battle phase, no pending battle, and an eligible standing front-row unit. |
| `MatchGame::attack_targets` | Lists occupied, unlocked opposing Vanguard/front rear-guard targets. |
| `MatchGame::boost_for` | Returns the selected attacker's eligible aligned booster. |
| `MatchGame::declare_attack` | Validates a player attack and opens its pending battle with the requested boost choice. |
| `MatchGame::can_guard` | Checks whether a selected hand card can guard the pending CPU attack. |
| `MatchGame::call_guardian` | Moves a valid selected hand card to the pending battle's Guardian Circle. |
| `MatchGame::can_intercept` | Checks whether a selected rear-guard can intercept the pending CPU attack. |
| `MatchGame::intercept` | Removes an eligible rear-guard from its circle and adds it as a guardian. |
| `MatchGame::player_guard_shield` | Allows legal guardian/intercept sources only during a CPU attack’s Guard step. |
| `MatchGame::add_player_guardian` | Validates and consumes a hand/intercept source, adds its card to guardians, logs the move, and returns success. |
| `tests::check_events_number_identical_twin_drive_cards_and_use_the_checking_side` | Checks unique IDs and attacker/defender ownership for identical Twin Drive cards and subsequent damage on both attacking sides. |
| `tests::rear_guard_attacks_publish_damage_without_a_drive_reveal` | Checks that a rear-guard hit publishes only the defender's damage event. |
| `MatchGame::resolve_pending_attack` | Advances one visible battle step: reveals checks, pauses for human trigger choices, applies CPU choices, determines hits and critical damage, then closes the battle. Pending choices cannot be bypassed. |
| `MatchGame::trigger_decision` | Returns the current required trigger part and revealed card name, if the human must choose. |
| `MatchGame::trigger_targets` | Lists legal human field targets for the current trigger part. |
| `MatchGame::choose_trigger_unit` | Validates a trigger target, applies its power/critical/Stand effect, clears that choice, and refreshes battle totals. |
| `MatchGame::choose_trigger_damage` | Heals a selected existing damage card, including face-down cards, and resumes the pending check. |
| `MatchGame::battle_totals` | Recalculates attack plus the current booster’s power, target power, and guardian shield from live unit state. |
| `MatchGame::refresh_battle_power` | Copies live attack and defense totals into the pending battle for display and decisions. |
| `MatchGame::can_restand` | Checks for the player's pending Dauntless-style Break Ride restand, a rested Vanguard, sufficient hand cost, and no unresolved attack. |
| `MatchGame::use_restand` | Pays the three-card discard, stands the Vanguard, clears the one-use restand flag, and logs the action. |
| `MatchGame::end_turn` | Ends Battle or the first global turn’s Main phase, clears temporary bonuses, unlocks the human rear-guards, increments the global turn, and starts CPU Stand. |
| `MatchGame::advance_cpu_turn` | Advances one timed CPU stage, drawing exactly once; skips Battle on global turn one and waits during unresolved attacks. |
| `MatchGame::advance_cpu_battle` | Chooses the next CPU attacker, handles the simplified re-stand opportunity, or advances to End. |
| `MatchGame::declare_cpu_attack` | Chooses a CPU attack target based on power and available opposing front units, then opens a boosted/unboosted pending battle. |
| `MatchGame::open_attack` | Validates attacker/target/booster, rests attacking participants, computes initial battle power, chooses CPU guardians when appropriate, and creates `PendingAttack`. |
| `MatchGame::start_player_turn` | Increments the global turn and enters human Stand without drawing or readying units ahead of their visible stages. |
| `MatchGame::advance_player_turn` | Commits standing units after their Stand animation, then draws once at the end of Draw and enters Ride. |
| `MatchGame::capture_outcome` | Ends the fight at six damage or empty deck after the current check/effects finish, and sends outstanding guardians to drop. |
| `pay_counter_blast` | Turns the requested number of available face-up damage cards face down, scanning in zone order. |
| `clear_turn_bonuses` | Clears every field unit’s temporary power and critical at turn boundaries for both fighters. |
| `trigger_unit_targets` | Lists unlocked field units for power/critical; Stand targets rear-guards only and Heal has no unit targets. |
| `apply_trigger_part` | Adds 5000 power, adds one critical, or stands the selected rear-guard. |
| `prepare_trigger_check` | Stages the checked card, queues matching-clan power and additional trigger parts, performs Draw’s extra draw, and checks Heal eligibility before the revealed card enters damage. |
| `ready_player` | Stands field units and clears temporary power bonuses at the start of their turn. |
| `unlock_rear_guards` | Clears locked state from all rear-guards. |
| `cpu_ride` | Chooses the highest eligible ride in the CPU hand and moves its previous Vanguard into soul. |
| `cpu_call` | Calls a high-power playable unit to the first empty rear-guard slot. |
| `cpu_limit_break` | Pays the CPU's simplified Counter-Blast cost, grants its Vanguard power, and performs the selected Link Joker lock behavior. |
| `restand_vanguard` | Discards the final three hand cards to drop and stands the Vanguard; callers must validate the cost first. |
| `GuardSource::shield` | Computes legal positive shield for a hand guardian or eligible grade-two front-row interceptor that is not the attack target. |
| `GuardSource::take_card` | Removes a validated hand/interceptor source and returns its card. |
| `auto_guard` | Selects enough eligible low-shield guardians/interceptors to defend, or passes if their total shield cannot meet the requirement. |
| `matches_clan` | Checks whether a card matches the clan of the player's Vanguard or any rear-guard. |
| `tests::card` | Creates a small Royal Paladin test card with specified grade, power, and shield. |
| `tests::unit` | Wraps a test card as an optional field unit. |
| `tests::battle` | Constructs a controlled Battle-phase fixture with predictable nonmatching-clan deck cards. |
| `tests::finish_attack` | Advances a bounded staged battle, explicitly selecting legal human trigger targets, until completion. |
| `tests::flipping_damage_updates_costs_without_moving_or_reordering_cards` | Checks manual flips, automatic cost availability, invalid indices, and preservation of the damage zone. |
| `tests::damage_cannot_be_manually_changed_during_opponent_turn_battle_or_result` | Rejects resource edits during pending attacks, CPU/finished phases, and completed results. |
| `tests::attack_targets_only_include_occupied_unlocked_front_row_units` | Rejects locked/back-row targets and verifies target eligibility. |
| `tests::each_column_uses_its_own_optional_booster` | Verifies alignment, optional boosting, and resting of the correct participants. |
| `tests::opponent_stages_advance_their_actions_once_and_wait_for_guarding` | Checks ordered CPU stages, one draw, and that repeated progression cannot bypass Guard. |
| `tests::each_cpu_attack_pauses_for_a_guard_decision` | Verifies individual CPU battle pauses, hand guardians, and resolution flow. |
| `tests::rested_front_rear_guard_can_intercept_for_another_unit` | Allows a rested eligible interceptor while preserving target restrictions. |
| `tests::drive_check_can_punch_through_a_guard_for_a_rear_guard_target` | Verifies that a drive-trigger power change can alter a guarded battle against a rear-guard. |
| `tests::stand_allows_another_rear_guard_attack_but_critical_does_not` | Distinguishes standing another attacker from adding critical to the current battle. |
| `tests::opening_draw_and_first_turn_skip_battle` | Checks the opening draw, blocked first Battle, direct Main-to-End transition, and exactly one next-turn draw. |
| `tests::normal_calls_include_triggers_and_retire_overcalled_units` | Checks legal trigger calls, overcall retirement, and that an invalid ride preserves the hand. |
| `tests::split_critical_power_updates_boost_total_and_expires_at_end` | Checks separate power/critical choices, a triggered booster’s live total, required-choice pauses, two damage checks, and bonus expiry. |
| `tests::sixth_damage_heal_pauses_cpu_and_can_remove_face_down_damage` | Checks sixth-damage Heal timing, human choice pauses during the CPU turn, and face-down damage removal. |
| `tests::draw_trigger_adds_one_card_and_stand_cannot_target_vanguard` | Checks Draw’s extra card and default shield, and excludes Vanguard from Stand targets. |
| `tests::damage_power_does_not_cancel_remaining_critical_damage` | Checks damage-trigger defense increases without interrupting the already-determined critical damage count. |
| `tests::empty_deck_loses_after_the_last_draw_resolves` | Checks deck-out after the final checked card reaches hand and pending battle cleanup. |
| `tests::rps_ties_replay_and_only_the_winner_chooses_order` | Covers all nine hand pairings, replaying ties and restricting the first/second choice to the winner. |
| `tests::mulligan_is_once_only_and_draws_before_returned_cards_are_shuffled` | Rejects invalid selections, replaces all five without redrawing returned copies, preserves deck size, and enforces one redraw/confirmation. |
| `tests::cpu_going_first_skips_battle_and_human_second_can_attack` | Checks CPU-first setup, first-turn Battle skipping, and the second fighter’s attack eligibility. |
| `tests::human_stand_precedes_draw_and_cannot_draw_twice` | Checks standing before Draw, exactly one card entering hand, and repeated progression not drawing again. |
| `tests::locked_rear_guard_unlocks_at_its_owners_end_step` | Keeps a human unit locked across CPU End and unlocks it at the human End. |

## `src/game/engine/opening.rs`

Source: [src/game/engine/opening.rs](../src/game/engine/opening.rs).

| Function | Responsibility |
| --- | --- |
| `RpsChoice::label` | Returns ROCK, PAPER, or SCISSORS for the opening choice. |
| `RpsChoice::beats` | Compares two hands using the three winning rock-paper-scissors pairs. |
| `MatchGame::play_rps` | Accepts one human/CPU hand pair in the correct setup stage and records a tie or winner for the reveal. |
| `MatchGame::finish_rps_reveal` | Replays ties, offers the human winner turn order, or applies the CPU winner’s policy of choosing first. |
| `MatchGame::choose_turn_order` | Allows the human RPS winner to choose first or second, then enters Mulligan. |
| `MatchGame::mulligan` | Validates a single redraw, replaces each opening hand in turn order, logs the CPU redraw count, and enters Hands Ready. |
| `MatchGame::confirm_opening_hands` | Confirms both five-card hands and enters the timed Stand Up presentation. |
| `MatchGame::begin_fight` | Starts global turn one with the selected first fighter in Stand after the opening presentation. |
| `valid_mulligan` | Rejects more than five indices, duplicates, and indices outside the hand before mutation. |
| `replace_opening_hand` | Returns selected cards to the deck bottom, draws the same number from the top, then shuffles if any were returned. |
| `cpu_mulligan` | Keeps one normal unit of each grade one through three and selects other opening cards for replacement. |

## `src/ui.rs`

Source: [src/ui.rs](../src/ui.rs).

| Function | Responsibility |
| --- | --- |
| `App::load` | Loads opening artwork/font, renders database/tabletop initialization progress across frames, creates UI state and the card-art resolver, and enters Title. |
| `App::draw` | Clears the frame, draws applicable ambient effects, and dispatches the active screen; Title input opens Menu. |
| `App::draw_menu` | Draws the shared opening scene and Story/Quick Fight/Deck Builder buttons; Escape returns to Title. |
| `App::draw_match_loading` | Renders stage progress, constructs the chapter fight with a saved custom player deck for Quick Fight when available, preloads visible opening art, handles cancellation, and enters Match after completion. |
| `App::draw_routes` | Draws school choices, applies a selected story route, and starts loading its first chapter. |
| `App::draw_match` | Sequences setup and timed human/CPU phases, pauses for check cut-ins and trigger/guard decisions, observes numbered reveal events, preloads art, renders live power and card motion, and routes legal controls. |
| `App::battlefield_highlights` | Highlights legal calls/overcalls, attacks, boosters, guards, trigger targets, and damage cards eligible for Heal. |
| `App::apply_match_action` | Handles log/menu without losing selection; clears other transient choices and dispatches the selected engine action. |
| `App::handle_match_clicks` | Handles rides/calls, resource flips, trigger choices, guardian click/drag input, attack targeting, and tapping an aligned booster to select or toggle it. |
| `App::is_dragging_guard` | Compares the cursor to its recorded guard press and reports whether it moved past the drag threshold. |
| `App::draw_result` | Presents win/loss, chapter summary/reward, replay/next chapter behavior, and a route back to Menu. |
| `App::draw_library` | Delegates to the full-window deck workshop and handles its Menu/Quick Fight navigation actions. |
| `App::start_match` | Clears the old fight, selections, log, and animated power state, then starts staged match loading. |
| `embedded_texture` | Decodes a bundled PNG into a linearly filtered Macroquad texture. |
| `field_unit_at` | Resolves a screen point to the Vanguard or a rear-guard selection on one side. |
| `mark_field_unit` | Writes a color highlight into the chosen fighter/circle entry. |
| `unit_name` | Returns a selected unit's card name, with a generic fallback. |
| `visible_card_names` | Collects visible hand/field/damage/guardian/check/drop art, plus the next human Draw card needed for its arrival flip. |
| `checked_card_name` | Removes the `Drive: ` or `Damage: ` prefix from a recorded check before card-art lookup. |
| `phase_delay` | Returns readable automatic stage durations: Stand 1.5 seconds, Draw 0.8, other CPU phases 0.65. |

## `src/ui/check_animation.rs`

Source: [src/ui/check_animation.rs](../src/ui/check_animation.rs).

| Function | Responsibility |
| --- | --- |
| `CheckAnimation::observe` | Starts a timed presentation exactly once for each new reveal sequence. |
| `CheckAnimation::active` | Reports whether the current normal/trigger cut-in still blocks battle progression and input. |
| `CheckAnimation::revealed_name` | Returns the current active card name for artwork preload. |
| `CheckAnimation::draw` | Composes the slanted frame, circuits, real zone/sleeve/card art, deck travel, flip/flash, fighter panel, check ribbons, trigger burst, and exit fade using elapsed time. |
| `duration` | Chooses 1.55 seconds for a normal card or 2.65 seconds for a trigger card. |
| `checking_fighter` | Resolves the checking side to the actual match name, using Aichi for the prototype's default You identity. |
| `cutin_frame` | Draws a four-point slanted metallic frame with side accent lines. |
| `quad` | Draws a colored quadrilateral from two triangles. |
| `draw_circuits` | Animates subdued circuit traces and hexagons behind the reveal. |
| `draw_zone_art` | Samples the existing real rear/Vanguard circle artwork into the focused check scene. |
| `draw_character` | Draws the checking fighter's official upper-body render, name, and side panel; falls back to a matching portrait or initial. |
| `check_ribbon` | Draws repeated English Drive/Damage Check captions, angled metallic backing, accent underline, and moving highlights. |
| `trigger_color` | Chooses gold Critical, red Draw, blue Stand, or green Heal. |
| `trigger_title` | Returns the English uppercase trigger name. |
| `secondary_label` | Describes the eligible secondary effect or explicitly reports an unmet Heal condition, missing Stand target, or no matching clan. |
| `draw_trigger` | Animates the trigger burst, brief colored flash, symbol, English announcement, and classic-era +5000 power/effect text. |
| `trigger_symbol` | Draws scalable Critical, Draw, Stand, and Heal symbols. |
| `outlined_text` | Draws rotated text with a dark outline for field readability. |
| `tests::reveal` | Constructs predictable numbered reveal fixtures. |
| `tests::identical_consecutive_checks_replay_once_per_sequence` | Checks that polling the same sequence does not restart it, while a new identical-card sequence does. |
| `tests::reveals_hold_resolution_until_the_full_cutin_finishes` | Checks that normal/trigger holds end at their intended duration. |
| `tests::trigger_copy_respects_clan_and_secondary_conditions` | Checks unavailable Heal/no-clan wording and all English trigger labels. |
| `tests::cutin_identity_uses_the_checking_fighter_not_the_attacker` | Checks default player, rival, custom player, and Reversed-rival identities. |

## `src/ui/deck_workshop.rs`

Source: [src/ui/deck_workshop.rs](../src/ui/deck_workshop.rs).

| Function | Responsibility |
| --- | --- |
| `DeckWorkshop::new` | Loads a saved list or initializes an empty draft and status; embeds the real logo, sleeve, and clan symbols. |
| `DeckWorkshop::quick_fight_deck` | Converts only the last saved list into a validated shuffled fight deck. |
| `DeckWorkshop::draw` | Applies a full-viewport logical camera and matching pointer transform, draws the workshop, and restores the default camera. |
| `DeckWorkshop::draw_content` | Coordinates keyboard/search/modal input, collection filtering, paging, art preload, grouped card rows, panels, status, and navigation actions. |
| `DeckWorkshop::draw_filters` | Draws nation dividers and clan symbols/names/counts; handles All/nation/clan selection and page reset. |
| `DeckWorkshop::draw_toolbar` | Draws top search, sort, multiple-grade toggles, and reset controls; manages field focus. |
| `DeckWorkshop::draw_sort_menu` | Draws the five-choice sort dropdown, handles selection, and blocks underlying input. |
| `DeckWorkshop::draw_deck` | Draws the paged unique-printing deck strip with quantities, loads clan samples, clears drafts, and handles inspection/removal. |
| `DeckWorkshop::draw_preview` | Draws the persistent selected-card art, metadata, ability excerpt, details action, and add/remove controls. |
| `DeckWorkshop::draw_statistics` | Shows grade bars, trigger counts, validation feedback, Save Deck, and Save & Fight; updates the saved list only after a successful write. |
| `catalog_rows` | Partitions sorted results into five-card rows, keeping nation/clan/grade boundaries in the default sort. |
| `nation_color` | Returns the six era nation accent colors. |
| `control` | Draws an enabled/disabled/selected logical-coordinate control and reports an eligible pointer click. |
| `badge` | Draws grade and quantity badges. |
| `clipped_text` | Truncates text to a measured width without splitting UTF-8 characters. |
| `thumbnail` | Draws cached real card art, with a visible missing-image fallback. |
| `plain_rules` | Converts the catalog's line-break and angle-bracket entities into readable preview text. |
| `draw_details` | Draws the modal full card image, metadata, printed rules, product, and flavor text. |
| `clan_icons` | Embeds and decodes all 23 wiki clan-symbol textures. |
| `tests::grouped_rows_keep_boundaries_and_every_card` | Checks row capacity, clan boundaries, and lossless pagination input. |

## `src/ui/presentation.rs`

Source: [src/ui/presentation.rs](../src/ui/presentation.rs).

| Function | Responsibility |
| --- | --- |
| `PresentationArt::new` | Decodes the embedded landscape, Aichi/Kai cutouts, logo, and title font. |
| `PresentationArt::font` | Returns a cloned font handle for consistent smooth text throughout the app. |
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
| `CardArt::from_layout` | Builds normalized-name and exact-printing-key mappings and candidate roots from explicitly supplied paths. |
| `CardArt::preload_records` | Preloads exact-printing keys for visible editor/deck/preview indices rather than matching by card name. |
| `CardArt::printing` | Returns the selected catalog printing's cached texture. |
| `CardArt::preload_keys` | Maintains the bounded texture cache and loads requested name or printing keys, caching unavailable assets. |
| `printing_key` | Builds a printing-specific cache key from the catalog card number. |
| `tests::editor_printings_resolve_their_exact_image_paths` | Checks that BT01 and BT15 Dragonic Overlord resolve different image paths and both decode. |
| `CardArt::preload` | Loads missing requested textures, remembers unavailable names, and evicts unrelated cached textures when approaching the cache limit. |
| `CardArt::textures` | Returns a read-only reference to the loaded texture map. |
| `CardArt::load_image` | Tries every matching printing against valid asset roots until one PNG decodes successfully. |
| `artwork_roots` | Finds distinct valid install/app-resource/Cargo-project/working-directory roots containing card assets. |
| `normalize_card_name` | Removes non-alphanumeric characters and lowercases card names for artwork lookup. |
| `tests::local_card_art_decodes_with_an_unrelated_working_directory` | Confirms that bundled card art can load through the project root without relying on the launch directory. |
| `tests::missing_printing_can_fall_back_to_another_printing` | Confirms that a missing printing can fall back to another path for the same normalized name. |

## `src/ui/fighters.rs`

Source: [src/ui/fighters.rs](../src/ui/fighters.rs).

| Function | Responsibility |
| --- | --- |
| `FighterArt::new` | Loads official embedded portraits and standing check cut-ins for Aichi, Kai, Morikawa, Kamui, Misaki, Ren, Asaka, Leon, Naoki, and Takuto. |
| `FighterArt::cutin` | Resolves a matching official standing render from the current fighter's whole-word name, including Reversed variants; returns none for unknown names. |
| `FighterArt::portrait` | Matches a fighter name by case-insensitive whole words, including names prefixed with Reversed; returns no portrait for unknown names. |

## `src/ui/board.rs`

Source: [src/ui/board.rs](../src/ui/board.rs).

| Function | Responsibility |
| --- | --- |
| `TabletopArt::sleeve` | Chooses the player's Aichi sleeve or opponent's Kai sleeve. |
| `BattlefieldLayout::new` | Fits a proportion-preserving logical tabletop into available screen bounds. |
| `BattlefieldLayout::point` | Projects a logical point into the perspective-tapered field. |
| `BattlefieldLayout::scale_at` | Returns the position-dependent scale shared by projected card rendering and hit geometry. |
| `BattlefieldLayout::rect` | Projects the logical center and scales the size at that center so render and hit bounds align. |
| `BattlefieldLayout::side_point` | Mirrors a player's local point across the tabletop for the opposing fighter. |
| `BattlefieldLayout::row_center` | Returns the logical center of an aligned front/back column for one fighter. |
| `BattlefieldLayout::rear_center` | Maps one of the five fixed rear-guard indices to its circle center. |
| `BattlefieldLayout::vanguard_center` | Returns the middle front-row circle center. |
| `BattlefieldLayout::guardian_center` | Returns the shared central Guardian Circle center. |
| `BattlefieldLayout::deck_rect` | Returns the shared rendered deck-card bounds used by both deck drawing and draw animation. |
| `BattlefieldLayout::check_rect` | Returns the checking side's actual Trigger Check Zone card destination for cut-in exit motion. |
| `BattlefieldLayout::damage_center` | Returns a fighter's damage-pocket center on the left of their field. |
| `BattlefieldLayout::damage_card` | Returns the sideways screen rectangle for an index in the vertically overlapping damage stack. |
| `BattlefieldLayout::damage_at` | Searches occupied damage cards from top to bottom in draw order, making both overlaps and exposed strips select the visible card. |
| `BattlefieldLayout::unit_at` | Tests a screen point against a circle and its card rectangle. |
| `BattlefieldLayout::rear_at` | Returns the rear-guard slot hit by a point. |
| `BattlefieldLayout::vanguard_at` | Reports whether a point hits the Vanguard circle/card. |
| `BattlefieldLayout::guardian_at` | Tests both halves of the full Guardian ellipse. |
| `BattlefieldLayout::label` | Draws a centered, scaled label at a logical position. |
| `draw_battlefield` | Composes the surface, side zones, damage stacks, circles, field units, soul counts, guardians, and hovered damage preview. |
| `draw_table_surface` | Draws the dark trapezoid arena, green hexagons, and subdued perimeter; no column connector lines. |
| `draw_attack_path` | Draws a gold direction arrow between the current attacker and target. |
| `draw_mat_art` | Samples and layers a transparent circle graphic at a field position with the appropriate tint/orientation. |
| `draw_circle_unit` | Draws a circle's interactive highlight and optional unit card. |
| `draw_guardians` | Draws a complete green Guardian ring and hit highlight, then distributes the pending battle's guardian cards across it. |
| `draw_damage_preview` | Places a full upright damage-card face and caption beside its stack, clamped to field bounds. |
| `draw_sleeve` | Draws a proportionally supplied card-back rectangle, rotating the opponent's normal deck/check view. Sideways damage backs use their own quarter-turn in `draw_damage`. |
| `draw_side_zones` | Draws metallic Deck/Drop/landscape Trigger frames, deck sleeves/counts, and sideways latest-check art. |
| `draw_zone_frame` | Draws a translucent zone with clipped corners and metallic border. |
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
| `draw_field_unit` | Draws full card art, grade, live power and critical; smoothly rotates rested units and moves their labels with the rotated bounds during Stand. |
| `draw_guard_card` | Draws a guardian card sideways using the common face renderer. |
| `draw_card_face` | Draws full card art or a clan/name fallback, preserves quarter-turn geometry when requested, and adds a clan-colored edge. |
| `draw_clan_art` | Draws a small vector emblem used when a card face is unavailable. |
| `HandLayout::new` | Calculates readable hand-card size, overlap spacing, page capacity, visible count, and a clamped page. |
| `HandLayout::during_draw` | Slides existing hand positions toward the next hand’s spacing while retaining the current card count. |
| `HandLayout::draw_destination` | Returns the incoming card’s final slot when it belongs to the current page. |
| `HandLayout::next_page_button` | Returns the next-page control’s bounds, also used for off-page card arrivals. |
| `HandLayout::visible_indices` | Returns the hand-index range on the active page. |
| `HandLayout::card_rect` | Returns a visible hand card's rectangle. |
| `HandLayout::exposed_width` | Returns how much of a hand card is visible before the next overlapping card. |
| `HandLayout::card_at` | Finds the visible hand card under a point, checking overlap in topmost order. |
| `HandLayout::draw_page_controls` | Draws previous/next controls when required and returns a requested page change when paging is enabled. |
| `draw_player_hand` | Draws the hand ribbon, playable/guard/selected highlights, captions and shield labels, plus an enlarged hovered card. |
| `hand_card_caption` | Formats the card name, grade, power and shield summary used by the hand. |
| `draw_shield_label` | Fits a positive shield indicator within the card's exposed hand segment. |
| `draw_hand_preview` | Draws a larger upright card preview near the hand panel. |
| `fit_caption` | Fits a label to available width, truncating with an ellipsis when necessary. |
| `tests::overlapping_hand_cards_use_the_visible_card_for_input` | Verifies that overlapping hand rectangles choose the exposed/topmost card. |
| `tests::every_hand_card_remains_reachable_when_the_ribbon_pages` | Verifies access to all hand cards across page ranges. |
| `tests::draw_reserves_the_same_slot_the_final_hand_uses` | Checks that shifted hand geometry and the arrival destination equal the committed hand layout at several hand sizes. |

## `src/ui/match_flow.rs`

Source: [src/ui/match_flow.rs](../src/ui/match_flow.rs).

| Function | Responsibility |
| --- | --- |
| `MatchPrompt::new` | Derives trigger/guard/attack/boost/phase decisions and live power previews; opening Main offers End Turn instead of Battle. |
| `MatchPrompt::choice` | Appends an available contextual action with its label and primary-button flag. |
| `tests::guard_decision_overrides_opponent_waiting_and_blocks_phase_actions` | Checks that a Guard decision replaces the CPU waiting prompt and excludes End Turn. |
| `tests::boost_prompt_has_both_choices_and_a_cancel` | Checks explicit boost/no-boost/cancel commands and the booster’s power hint. |
| `tests::empty_ride_hand_offers_an_explicit_next_step` | Checks Skip Ride as the primary action when the hand has no legal ride. |
| `tests::opening_main_offers_end_turn_instead_of_battle` | Checks that opening Main offers End Turn and no Enter Battle command. |

## `src/ui/match_hud.rs`

Source: [src/ui/match_hud.rs](../src/ui/match_hud.rs).

| Function | Responsibility |
| --- | --- |
| `MatchLayout::new` | Allocates the large field, floating upper-left phase list, corner fighter bars, bottom hand, compact right commands, power meter, and optional log. |
| `MatchLayout::blocks_field_input` | Prevents clicks on overlay controls or status from selecting a field card underneath. |
| `draw_match_shell` | Draws quiet dark-green hexagons behind the field and overlays. |
| `draw_match_hud` | Draws turn ownership/number and fighter bars with the selected official portraits. |
| `fighter_bar` | Draws a compact corner portrait, fighter name, damage, icon counts, and available Counter-Blast. |
| `draw_resource_count` | Draws a deck stack or fanned-hand icon alongside its numeric count. |
| `draw_phase_track` | Draws the floating six-stage list with the active phase enlarged, brightened, and weighted. |
| `draw_decision_banner` | Shows a short instruction for an active selection or trigger; hides routine phase narration. |
| `PowerDisplay::draw` | Smoothly animates attack and defense-plus-shield meters from live battle or selected-boost totals, fading back to zero after battle. |
| `draw_action_panel` | Draws only the current decision choices and available abilities, plus compact log/menu controls. |
| `draw_battle_log` | Draws a sized overlay of recent log entries, emphasizing the latest entry. |
| `draw_command_button` | Draws a command button's primary/secondary/hover appearance and reports a click. |
| `beveled_panel` | Draws a clipped-corner polygon panel and border. |
| `draw_left_lines` | Wraps short overlay instructions into a bounded number of lines. |

## `src/ui/match_setup.rs`

Source: [src/ui/match_setup.rs](../src/ui/match_setup.rs).

| Function | Responsibility |
| --- | --- |
| `App::draw_setup` | Draws paired RPS choices, turn-order controls, the five-card redraw selector, and hand readiness; routes their inputs and reveal timing. |
| `setup_heading` | Draws the responsive opening-screen title and short instruction. |
| `sign_tile` | Draws a hand-choice tile with selected/hover states and returns a legal click. |
| `draw_hand_sign` | Draws original vector silhouettes for Rock, Paper, and Scissors. |
| `draw_stand_up` | Fades the Stand Up banner in and out over the ready field. |
| `draw_draw_animation` | Moves a proportionate card from the shared deck rectangle into the exact future hand slot, with eased motion and a late face flip. Off-page draws shrink into the paging control. CPU draws stay hidden. |

## `src/ui/motion.rs`

Source: [src/ui/motion.rs](../src/ui/motion.rs).

| Function | Responsibility |
| --- | --- |
| `smoothstep` | Clamps and eases a zero-to-one value with zero endpoint velocity. |
| `draw_progress` | Maps Draw phase time to a short eased travel interval after an initial pause. |
| `interpolate_rect` | Interpolates card position and size between exact source and destination bounds. |

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

## `scripts/fetch_clan_icons.py`

Source: [scripts/fetch_clan_icons.py](../scripts/fetch_clan_icons.py).

| Function | Responsibility |
| --- | --- |
| `fetch` | Downloads a clan's original PNG via MediaWiki filename-hash paths, verifies the image format, preserves its bytes, and returns provenance/hash/dimension metadata. |
| `main` | Finds all catalog clans, fetches their symbols with bounded concurrency, and writes the source manifest. |

## `scripts/build_branding.py`

Source: [scripts/build_branding.py](../scripts/build_branding.py).

| Function | Responsibility |
| --- | --- |
| `artwork` | Reads a credited source image as RGBA. |
| `fitted` | Fits a source image inside a target size with Lanczos resampling. |
| `place` | Composites a source layer and optional alpha-derived shadow. |
| `english_wordmark` | Isolates the English component of the official media-kit mark while omitting its Japanese subtitle. |
| `app_icon` | Composes the transparent circular 1024 px icon from real character, circle, clan, and wordmark assets. |
| `game_logo` | Composes the transparent English title mark with era subtitle and clan symbols. |
| `export_icons` | Exports PNG sizes, raw runtime RGBA, ICO, Apple iconset, and ICNS. |
| `manifest` | Records non-AI source provenance, transformations, and input/output checksums. |
| `main` | Regenerates the complete branding set deterministically. |

## `scripts/build_macos_app.py`

Source: [scripts/build_macos_app.py](../scripts/build_macos_app.py).

| Function | Responsibility |
| --- | --- |
| `main` | Builds release/debug on macOS, packages the executable, ICNS and external card cache, writes the application metadata, and reports the resulting app path. |
