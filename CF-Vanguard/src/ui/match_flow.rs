//! Match decisions and available actions, derived without rendering or mutating play.

use super::drawing::{CYAN, GOLD, RED};
use super::{unit_name, AttackSelection, GuardSelection, MatchAction};
use crate::game::engine::{AttackUnit, BattleStep, MatchGame, Phase, TriggerPart};
use macroquad::prelude::Color;

pub(super) struct ActionChoice {
    pub label: &'static str,
    pub action: MatchAction,
    pub primary: bool,
}

pub(super) struct MatchPrompt {
    pub tag: &'static str,
    pub step: &'static str,
    pub title: String,
    pub detail: String,
    pub accent: Color,
    pub choices: Vec<ActionChoice>,
    pub combat: Option<(i32, i32, i32)>,
}

impl MatchPrompt {
    pub(super) fn new(
        game: &MatchGame,
        attack_selection: Option<AttackSelection>,
        selected_hand: Option<usize>,
        guard_selection: Option<GuardSelection>,
    ) -> Self {
        let mut prompt = Self {
            tag: "YOUR DECISION",
            step: "",
            title: String::new(),
            detail: String::new(),
            accent: CYAN,
            choices: Vec::new(),
            combat: None,
        };
        if let Some((part, name)) = game.trigger_decision() {
            prompt.step = if game.pending_attack.as_ref().unwrap().step == BattleStep::Drive {
                "DRIVE TRIGGER"
            } else {
                "DAMAGE TRIGGER"
            };
            prompt.title = part.instruction().into();
            prompt.detail = format!(
                "{} · {}",
                name,
                if part == TriggerPart::Heal {
                    "Click a damage card's exposed strip. Face-down damage can be healed too."
                } else {
                    "Click a highlighted unit on your field to apply the trigger."
                }
            );
            prompt.combat = game.battle_totals();
            return prompt;
        }
        if let Some(attack) = &game.pending_attack {
            let (offense, defense) = if attack.cpu_attacking {
                (&game.cpu, &game.player)
            } else {
                (&game.player, &game.cpu)
            };
            if attack.step != BattleStep::Guard {
                prompt.tag = "AUTOMATIC CHECK";
                prompt.step = match attack.step {
                    BattleStep::Drive => "DRIVE STEP",
                    BattleStep::Damage => "DAMAGE STEP",
                    BattleStep::Close => "CLOSE STEP",
                    BattleStep::Guard => unreachable!(),
                };
                prompt.title = game.message.clone();
                prompt.detail =
                    "Checks advance automatically. Trigger selections will pause the battle."
                        .into();
                prompt.combat = game.battle_totals();
                return prompt;
            }
            prompt.step = "GUARD STEP";
            prompt.title = if attack.cpu_attacking {
                "Choose how to guard".into()
            } else {
                "Opponent's guard is ready".into()
            };
            prompt.detail = format!(
                "{} attacks {}. {}",
                unit_name(attack.attacker, offense),
                unit_name(attack.target, defense),
                if attack.cpu_attacking {
                    if guard_selection.is_some() {
                        "Place the selected guardian on the green Guardian Circle."
                    } else {
                        "Choose a highlighted hand card or interceptor, or finish with the action on the right."
                    }
                } else {
                    "Resolve the attack to run checks and determine the result."
                }
            );
            prompt.combat = Some((attack.attack_power, attack.target_power, attack.shield()));
            prompt.choice(
                if !attack.cpu_attacking {
                    "RESOLVE ATTACK"
                } else if attack.guardians.is_empty() {
                    "NO GUARD"
                } else {
                    "DONE GUARDING"
                },
                MatchAction::Resolve,
                true,
            );
            if guard_selection.is_some() {
                prompt.choice("CANCEL SELECTION", MatchAction::CancelSelection, false);
            }
            return prompt;
        }
        if let Some(selection) = attack_selection {
            match selection {
                AttackSelection::Target { attacker, boost } => {
                    prompt.step = "ATTACK STEP / TARGET";
                    prompt.title =
                        format!("Choose a target for {}", unit_name(attacker, &game.player));
                    prompt.detail =
                        "Select the opposing Vanguard or a highlighted front-row rear-guard."
                            .into();
                    prompt.detail = if boost {
                        "Boost selected. Tap the booster to remove it; choose an opposing target."
                    } else {
                        "Choose a target. Tap the unit behind your attacker to add its boost."
                    }
                    .into();
                    prompt.combat = Some((
                        attacker.unit(&game.player).map_or(0, |u| u.power())
                            + if boost {
                                game.boost_for(attacker).map_or(0, |u| u.power())
                            } else {
                                0
                            },
                        0,
                        0,
                    ));
                    prompt.accent = GOLD;
                }
                AttackSelection::Boost { attacker, target } => {
                    prompt.step = "ATTACK STEP / BOOST";
                    prompt.title = format!("Boost {}?", unit_name(attacker, &game.player));
                    prompt.detail = game.boost_for(attacker).map_or_else(
                        || "Continue without a boost.".into(),
                        |unit| {
                            format!(
                                "{} can add {} power. Choose Boost or No Boost on the right.",
                                unit.card.name,
                                unit.power()
                            )
                        },
                    );
                    prompt.combat = Some((
                        attacker.unit(&game.player).map_or(0, |u| u.power())
                            + game.boost_for(attacker).map_or(0, |u| u.power()),
                        target.unit(&game.cpu).map_or(0, |u| u.power()),
                        0,
                    ));
                    prompt.choice(
                        "BOOST",
                        MatchAction::Declare {
                            attacker,
                            target,
                            boost: true,
                        },
                        true,
                    );
                    prompt.choice(
                        "NO BOOST",
                        MatchAction::Declare {
                            attacker,
                            target,
                            boost: false,
                        },
                        false,
                    );
                }
            }
            prompt.choice("CANCEL ATTACK", MatchAction::CancelSelection, false);
            return prompt;
        }
        match game.phase {
            Phase::Setup | Phase::Stand | Phase::Draw => {
                prompt.tag = "AUTOMATIC PHASE";
                prompt.step = "";
            }
            Phase::Ride => {
                prompt.step = "RIDE PHASE";
                let can_ride = !game.ride_options().is_empty();
                prompt.title = if can_ride {
                    "Ride your next Vanguard"
                } else {
                    "No legal ride in hand"
                }
                .into();
                prompt.detail = if can_ride {
                    "Choose a highlighted card in your hand to ride, or skip to Main phase."
                } else {
                    "Choose Skip Ride on the right to continue to Main phase."
                }
                .into();
                prompt.choice("SKIP RIDE", MatchAction::SkipRide, !can_ride);
            }
            Phase::Call => {
                prompt.step = "MAIN PHASE";
                if let Some(card) = selected_hand.and_then(|i| game.player.hand.get(i)) {
                    prompt.title = "Choose a rear-guard circle".into();
                    prompt.detail = format!(
                        "Call {} to a highlighted circle. An occupied unit will be retired.",
                        card.name
                    );
                } else {
                    prompt.title = "Build your field".into();
                    prompt.detail = "Select a card, then a rear-guard circle. An occupied unit will be retired.".into();
                }
                if game.turn == 1 {
                    prompt.choice("END TURN", MatchAction::EndTurn, true);
                } else {
                    prompt.choice("ENTER BATTLE", MatchAction::Battle, true);
                }
                if selected_hand.is_some() {
                    prompt.choice("CANCEL SELECTION", MatchAction::CancelSelection, false);
                }
            }
            Phase::Battle => {
                prompt.step = "BATTLE PHASE / ATTACK";
                let can_attack = AttackUnit::FRONT_ROW
                    .into_iter()
                    .any(|unit| game.can_attack(unit));
                prompt.title = if game.turn == 1 {
                    "No attack on the first turn"
                } else if can_attack {
                    "Choose your attacker"
                } else {
                    "No standing attackers"
                }
                .into();
                prompt.detail = if game.turn == 1 { "The starting fighter cannot attack. End your turn to pass play to the opponent." } else if can_attack {
                    "Select a highlighted front-row unit, then choose its target. You can end the turn on the right."
                } else {
                    "Use an available ability or end your turn to pass play to the opponent."
                }.into();
                prompt.choice("END TURN", MatchAction::EndTurn, !can_attack);
            }
            Phase::Cpu => {
                prompt.tag = "OPPONENT'S TURN";
                prompt.step = "AUTOMATIC PROGRESSION";
                prompt.accent = RED;
                prompt.title = format!("{} phase", game.stage().unwrap().label());
                prompt.detail = format!(
                    "{} is playing. The game will pause here when you need to guard.",
                    game.cpu.name
                );
            }
            Phase::Finished => {
                prompt.tag = "FIGHT COMPLETE";
                prompt.step = "RESULT";
                prompt.accent = GOLD;
                prompt.title = if game.winner == Some(true) {
                    "Victory!"
                } else {
                    "Cardfight finished"
                }
                .into();
                prompt.detail = game.message.clone();
                prompt.choice("VIEW RESULT", MatchAction::Result, true);
            }
        }
        prompt
    }

    fn choice(&mut self, label: &'static str, action: MatchAction, primary: bool) {
        self.choices.push(ActionChoice {
            label,
            action,
            primary,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::card::{Card, Unit};
    use crate::story::CHAPTERS;

    #[test]
    fn guard_decision_overrides_opponent_waiting_and_blocks_phase_actions() {
        let mut game = MatchGame::new(CHAPTERS[0]);
        game.phase = Phase::Battle;
        game.end_turn();
        for _ in 0..5 {
            game.advance_cpu_turn();
        }
        assert!(game.pending_attack.is_some());
        let prompt = MatchPrompt::new(&game, None, None, None);
        assert_eq!(prompt.tag, "YOUR DECISION");
        assert_eq!(prompt.step, "GUARD STEP");
        assert_eq!(prompt.choices[0].action, MatchAction::Resolve);
        assert!(!prompt
            .choices
            .iter()
            .any(|c| c.action == MatchAction::EndTurn));
        assert!(prompt.combat.is_some());
    }

    #[test]
    fn boost_prompt_has_both_choices_and_a_cancel() {
        let mut game = MatchGame::new(CHAPTERS[0]);
        game.phase = Phase::Battle;
        game.player.rear[2] = Some(Unit::new(Card::unit(
            "Booster",
            "Royal Paladin",
            1,
            7000,
            5000,
        )));
        let prompt = MatchPrompt::new(
            &game,
            Some(AttackSelection::Boost {
                attacker: AttackUnit::Vanguard,
                target: AttackUnit::Vanguard,
            }),
            None,
            None,
        );
        assert_eq!(prompt.choices.len(), 3);
        assert!(prompt.detail.contains("7000"));
        assert!(matches!(
            prompt.choices[0].action,
            MatchAction::Declare { boost: true, .. }
        ));
        assert!(matches!(
            prompt.choices[1].action,
            MatchAction::Declare { boost: false, .. }
        ));
        assert_eq!(prompt.choices[2].action, MatchAction::CancelSelection);
    }

    #[test]
    fn empty_ride_hand_offers_an_explicit_next_step() {
        let mut game = MatchGame::new(CHAPTERS[0]);
        game.phase = Phase::Ride;
        game.player.hand.clear();
        let prompt = MatchPrompt::new(&game, None, None, None);
        assert_eq!(prompt.title, "No legal ride in hand");
        assert_eq!(prompt.choices[0].action, MatchAction::SkipRide);
        assert!(prompt.choices[0].primary);
    }
    #[test]
    fn opening_main_offers_end_turn_instead_of_battle() {
        let mut game = MatchGame::new(CHAPTERS[0]);
        game.phase = Phase::Ride;
        game.skip_ride();
        let prompt = MatchPrompt::new(&game, None, None, None);
        assert!(prompt
            .choices
            .iter()
            .any(|c| c.action == MatchAction::EndTurn));
        assert!(!prompt
            .choices
            .iter()
            .any(|c| c.action == MatchAction::Battle));
    }
}
