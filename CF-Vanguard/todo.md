# Gameplay TODO

## Next: automatic battle resolution and turn completion

Requested follow-up; these behavior changes are **pending**.

- [ ] Automatically resolve an attack after the guard decision is complete. A hit should proceed through its damage checks without a separate **Resolve Attack** confirmation.
- [ ] Show check-resolution choices only for a **Vanguard swing's drive checks** or **damage triggers**. A rear-guard swing does not have a drive check. Pause for actual trigger choices; ordinary checks should advance automatically.
- [ ] Automatically end the turn after all attacks are finished. Account for Stand triggers and available re-stand abilities before deciding that there are no attacks left.
- [ ] Keep required human guarding and trigger-target choices interactive; resume automatically after those choices.

### Highlighted functions to revisit

| Area | Functions | Current role / follow-up |
| --- | --- | --- |
| **Trigger checks** | **[`MatchGame::resolve_pending_attack`](src/game/engine.rs#L492)**, **[`prepare_trigger_check`](src/game/engine.rs#L1052)** | Advance Guard → Drive → Damage → Close, reveal cards, and queue trigger effects. Use these to distinguish Vanguard drive checks from rear-guard attacks and damage checks. |
| **Trigger-choice pauses** | **[`MatchGame::trigger_decision`](src/game/engine.rs#L697)**, **[`choose_trigger_unit`](src/game/engine.rs#L708)**, **[`choose_trigger_damage`](src/game/engine.rs#L729)** | Expose a required choice and resume the battle after a unit or damage card is selected. |
| **UI pauses and timing** | **[`App::draw_match`](src/ui.rs#L300)**, **[`App::apply_match_action`](src/ui.rs#L548)**, **[`App::handle_match_clicks`](src/ui.rs#L594)** | Pace automatic checks, dispatch Resolve/End Turn, and handle guarding and trigger choices. |
| **Resolve button** | **[`MatchPrompt::new`](src/ui/match_flow.rs#L25)**, **[`draw_action_panel`](src/ui/match_hud.rs#L327)** | Build and draw the current Resolve Attack / No Guard / Done Guarding actions. Revise the extra resolve confirmation here. |
| **End turn** | **[`MatchGame::end_turn`](src/game/engine.rs#L791)**, **[`advance_cpu_turn`](src/game/engine.rs#L807)**, **[`advance_cpu_battle`](src/game/engine.rs#L850)**, **[`start_player_turn`](src/game/engine.rs#L962)** | Transfer turn ownership, sequence CPU attacks and phases, and perform the next Stand/Draw. Add the human all-attacks-finished transition around these methods. |

When implementing, test a rear-guard hit/miss, Vanguard single/Twin Drive, damage triggers, a Stand trigger granting another attack, a re-stand ability, and a final attack that ends the match.
