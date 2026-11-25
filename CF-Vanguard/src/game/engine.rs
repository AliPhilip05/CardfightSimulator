//! Turn state and cardfight actions. UI code calls these methods; this module does no drawing or I/O.

use super::card::{Card, DamageCard, Player, Trigger, Unit};
use super::deck::{build_deck, prepare_player};
use crate::story::Chapter;
mod opening;
pub use opening::{RpsChoice, SetupStage};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Setup,
    Stand,
    Draw,
    Ride,
    Call,
    Battle,
    Cpu,
    Finished,
}

/// Actual turn progression; automatic opponent steps advance individually.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TurnStage {
    Stand,
    Draw,
    Ride,
    Main,
    Battle,
    End,
}

impl TurnStage {
    pub const ALL: [Self; 6] = [
        Self::Stand,
        Self::Draw,
        Self::Ride,
        Self::Main,
        Self::Battle,
        Self::End,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Stand => "STAND",
            Self::Draw => "DRAW",
            Self::Ride => "RIDE",
            Self::Main => "MAIN",
            Self::Battle => "BATTLE",
            Self::End => "END",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AttackUnit {
    Vanguard,
    Rear(usize),
}

impl AttackUnit {
    pub(crate) const FRONT_ROW: [Self; 3] = [Self::Rear(1), Self::Vanguard, Self::Rear(3)];
    const CPU_ATTACK_ORDER: [Self; 3] = [Self::Rear(1), Self::Rear(3), Self::Vanguard];

    pub fn boost_slot(self) -> Option<usize> {
        match self {
            Self::Rear(1) => Some(0),
            Self::Vanguard => Some(2),
            Self::Rear(3) => Some(4),
            Self::Rear(_) => None,
        }
    }

    pub fn unit(self, player: &Player) -> Option<&Unit> {
        match self {
            Self::Vanguard => player.vanguard.as_ref(),
            Self::Rear(slot) => player.rear.get(slot)?.as_ref(),
        }
    }

    fn unit_mut(self, player: &mut Player) -> Option<&mut Unit> {
        match self {
            Self::Vanguard => player.vanguard.as_mut(),
            Self::Rear(slot) => player.rear.get_mut(slot)?.as_mut(),
        }
    }

    fn can_attack(self, player: &Player) -> bool {
        self.can_be_targeted(player) && self.unit(player).is_some_and(|unit| !unit.rested)
    }

    fn can_be_targeted(self, player: &Player) -> bool {
        self.boost_slot().is_some() && self.unit(player).is_some_and(|unit| !unit.locked)
    }

    fn booster(self, player: &Player) -> Option<&Unit> {
        let unit = player.rear.get(self.boost_slot()?)?.as_ref()?;
        (!unit.rested && !unit.locked && unit.card.grade <= 1).then_some(unit)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BattleStep {
    Guard,
    Drive,
    Damage,
    Close,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TriggerPart {
    Power,
    Critical,
    Stand,
    Heal,
}

impl TriggerPart {
    pub fn instruction(self) -> &'static str {
        match self {
            Self::Power => "Choose a unit for +5000 power",
            Self::Critical => "Choose a unit for +1 critical",
            Self::Stand => "Choose a rear-guard to stand",
            Self::Heal => "Choose a damage card to heal",
        }
    }
}

pub struct PendingAttack {
    pub cpu_attacking: bool,
    pub attacker: AttackUnit,
    pub target: AttackUnit,
    pub booster: Option<usize>,
    pub attack_power: i32,
    pub target_power: i32,
    pub guardians: Vec<Card>,
    pub step: BattleStep,
    pub checked_card: Option<Card>,
    pub decision: Option<TriggerPart>,
    trigger_parts: Vec<TriggerPart>,
    remaining_drive: usize,
    remaining_damage: usize,
    damage_started: bool,
    damage_dealt: usize,
    hit: bool,
    target_name: &'static str,
}

impl PendingAttack {
    fn checking_cpu(&self) -> bool {
        if self.step == BattleStep::Drive {
            self.cpu_attacking
        } else {
            !self.cpu_attacking
        }
    }

    pub fn shield(&self) -> i32 {
        self.guardians.iter().map(|card| card.shield).sum()
    }
}

pub struct MatchGame {
    pub player: Player,
    pub cpu: Player,
    pub phase: Phase,
    pub turn: u32,
    pub setup_stage: SetupStage,
    pub rps_result: Option<(RpsChoice, RpsChoice)>,
    pub player_won_rps: Option<bool>,
    pub player_first: bool,
    pub chapter: Chapter,
    pub message: String,
    pub log: Vec<String>,
    pub winner: Option<bool>,
    pub player_calls: u8,
    pub pending_attack: Option<PendingAttack>,
    break_ride_restand: bool,
    cpu_attack_index: usize,
    cpu_stage: TurnStage,
}

impl MatchGame {
    pub fn new(chapter: Chapter) -> Self {
        let player = prepare_player(
            "You",
            build_deck(chapter.player_clan, chapter.player_boss, chapter.break_ride),
        );
        let cpu = prepare_player(
            chapter.rival,
            build_deck(chapter.rival_clan, chapter.rival_boss, true),
        );
        let mut game = Self {
            player,
            cpu,
            phase: Phase::Setup,
            turn: 1,
            setup_stage: SetupStage::RockPaperScissors,
            rps_result: None,
            player_won_rps: None,
            player_first: true,
            chapter,
            message: "Your opening hand is ready. Choose a legal ride.".into(),
            log: Vec::new(),
            winner: None,
            player_calls: 0,
            pending_attack: None,
            break_ride_restand: false,
            cpu_attack_index: 0,
            cpu_stage: TurnStage::Stand,
        };
        game.log("Choose rock, paper, or scissors to decide turn order.");
        game
    }

    pub fn log(&mut self, line: impl Into<String>) {
        self.log.push(line.into());
        if self.log.len() > 8 {
            self.log.remove(0);
        }
    }

    pub fn stage(&self) -> Option<TurnStage> {
        match self.phase {
            Phase::Setup => None,
            Phase::Stand => Some(TurnStage::Stand),
            Phase::Draw => Some(TurnStage::Draw),
            Phase::Ride => Some(TurnStage::Ride),
            Phase::Call => Some(TurnStage::Main),
            Phase::Battle => Some(TurnStage::Battle),
            Phase::Cpu => Some(self.cpu_stage),
            Phase::Finished => None,
        }
    }

    pub fn ride_options(&self) -> Vec<usize> {
        let grade = self.player.grade();
        self.player
            .hand
            .iter()
            .enumerate()
            .filter(|(_, card)| card.grade >= grade && card.grade <= grade + 1 && card.grade <= 3)
            .map(|(index, _)| index)
            .collect()
    }

    pub fn call_options(&self) -> Vec<usize> {
        self.player
            .hand
            .iter()
            .enumerate()
            .filter(|(_, card)| card.grade <= self.player.grade())
            .map(|(index, _)| index)
            .collect()
    }

    pub fn ride(&mut self, hand_index: usize) {
        if self.phase != Phase::Ride || !self.ride_options().contains(&hand_index) {
            return;
        }
        let card = self.player.hand.remove(hand_index);
        let Some(previous) = self.player.vanguard.take() else {
            return;
        };
        let did_break_ride = previous.card.break_ride
            && self.player.damage.len() >= 4
            && previous.card.clan == card.clan;
        self.break_ride_restand = did_break_ride && previous.card.name == "Dauntless Drive Dragon";
        if did_break_ride {
            self.player.soul.push(previous.card);
            self.player.vanguard = Some(Unit {
                card: card.clone(),
                rested: false,
                power_bonus: 10000,
                critical_bonus: 0,
                locked: false,
            });
            self.message = format!("BREAK RIDE! {} gets +10000 power.", card.name);
        } else {
            self.player.soul.push(previous.card);
            self.player.vanguard = Some(Unit::new(card.clone()));
            self.message = format!("You rode {}.", card.name);
        }
        self.log(self.message.clone());
        self.phase = Phase::Call;
    }

    pub fn skip_ride(&mut self) {
        if self.phase == Phase::Ride {
            self.phase = Phase::Call;
            self.message = "Ride phase complete. Call your rear-guards.".into();
        }
    }

    pub fn call(&mut self, hand_index: usize, rear_slot: usize) {
        if self.phase != Phase::Call || rear_slot >= 5 || hand_index >= self.player.hand.len() {
            return;
        }
        let card = self.player.hand.remove(hand_index);
        if card.grade > self.player.grade() {
            self.player.hand.insert(hand_index, card);
            return;
        }
        let name = card.name;
        if let Some(previous) = self.player.rear[rear_slot].take() {
            self.player.drop_zone.push(previous.card);
        }
        self.player.rear[rear_slot] = Some(Unit::new(card));
        self.player_calls += 1;
        self.message = format!("Called {name} to circle {}.", rear_slot + 1);
        self.log(self.message.clone());
    }

    pub fn begin_battle(&mut self) {
        if self.phase == Phase::Call && self.turn > 1 {
            self.phase = Phase::Battle;
            self.message = "Choose a standing front-row unit to attack.".into();
        }
    }

    pub fn can_limit_break(&self) -> bool {
        matches!(self.phase, Phase::Call | Phase::Battle)
            && self.pending_attack.is_none()
            && self
                .player
                .vanguard
                .as_ref()
                .map_or(false, |unit| unit.card.limit_break)
            && self.player.damage.len() >= 4
            && self.player.face_up_damage() >= 2
    }

    /// Manual tabletop resource control. Automatic ability costs read the same
    /// face-up state, so spent damage cannot be paid a second time.
    pub fn can_flip_damage(&self) -> bool {
        matches!(self.phase, Phase::Ride | Phase::Call | Phase::Battle)
            && self.pending_attack.is_none()
            && self.winner.is_none()
    }

    pub fn flip_damage(&mut self, index: usize) -> bool {
        if !self.can_flip_damage() {
            return false;
        }
        let Some(damage) = self.player.damage.get_mut(index) else {
            return false;
        };
        damage.face_up = !damage.face_up;
        self.message = if damage.face_up {
            format!(
                "Damage {} turned face up (manual Counter-Charge / correction).",
                index + 1
            )
        } else {
            format!("Counter-Blast 1: damage {} turned face down.", index + 1)
        };
        self.log(self.message.clone());
        true
    }

    pub fn use_limit_break(&mut self) {
        if !self.can_limit_break() {
            return;
        }
        pay_counter_blast(&mut self.player, 2);
        let name = self.player.vanguard.as_ref().unwrap().card.name;
        if name.contains("Nebula Lord") {
            if let Some(slot) = [0usize, 2, 4]
                .into_iter()
                .find(|slot| self.cpu.rear[*slot].is_some())
            {
                if let Some(unit) = self.cpu.rear[slot].as_mut() {
                    unit.locked = true;
                }
            }
            let locked = self
                .cpu
                .rear
                .iter()
                .flatten()
                .filter(|unit| unit.locked)
                .count() as i32;
            if let Some(vanguard) = self.player.vanguard.as_mut() {
                vanguard.power_bonus += 3000 * locked;
            }
            for slot in [1usize, 3] {
                if let Some(unit) = self.player.rear[slot].as_mut() {
                    if unit.card.clan == "Link Joker" {
                        unit.power_bonus += 3000 * locked;
                    }
                }
            }
            self.message = "Nebula Lord locks an opposing back-row unit.".into();
        } else if name.contains("Vowing Saber") {
            let mut locked = 0;
            for unit in self.player.rear.iter_mut().flatten() {
                if !unit.locked && unit.card.name.contains("Eradicator") && locked < 2 {
                    unit.locked = true;
                    locked += 1;
                }
            }
            let mut retired = 0;
            for unit in self.cpu.rear.iter_mut() {
                if unit.is_some() && retired < 2 {
                    if let Some(unit) = unit.take() {
                        self.cpu.drop_zone.push(unit.card);
                    }
                    retired += 1;
                }
            }
            if let Some(vanguard) = self.player.vanguard.as_mut() {
                vanguard.power_bonus += 10000;
            }
            self.message = "Vowing Saber Reverse retires two enemy rear-guards.".into();
        } else {
            if let Some(vanguard) = self.player.vanguard.as_mut() {
                vanguard.power_bonus += 10000;
            }
            self.message = "Limit Break! Your Vanguard gains +10000 power.".into();
        }
        self.log(self.message.clone());
    }

    pub fn can_attack(&self, attacker: AttackUnit) -> bool {
        self.phase == Phase::Battle
            && self.turn > 1
            && self.pending_attack.is_none()
            && attacker.can_attack(&self.player)
    }

    pub fn attack_targets(&self) -> Vec<AttackUnit> {
        AttackUnit::FRONT_ROW
            .into_iter()
            .filter(|target| target.can_be_targeted(&self.cpu))
            .collect()
    }

    pub fn boost_for(&self, attacker: AttackUnit) -> Option<&Unit> {
        attacker.booster(&self.player)
    }

    pub fn declare_attack(
        &mut self,
        attacker: AttackUnit,
        target: AttackUnit,
        use_boost: bool,
    ) -> bool {
        if !self.can_attack(attacker) {
            return false;
        }
        self.open_attack(false, attacker, target, use_boost)
    }

    pub fn can_guard(&self, hand_index: usize) -> bool {
        self.player_guard_shield(GuardSource::Hand(hand_index))
            .is_some()
    }

    pub fn call_guardian(&mut self, hand_index: usize) -> bool {
        self.add_player_guardian(GuardSource::Hand(hand_index))
    }

    pub fn can_intercept(&self, slot: usize) -> bool {
        self.player_guard_shield(GuardSource::Intercept(slot))
            .is_some()
    }

    pub fn intercept(&mut self, slot: usize) -> bool {
        self.add_player_guardian(GuardSource::Intercept(slot))
    }

    fn player_guard_shield(&self, source: GuardSource) -> Option<i32> {
        let attack = self.pending_attack.as_ref()?;
        if !attack.cpu_attacking || attack.step != BattleStep::Guard {
            return None;
        }
        source.shield(&self.player, attack.target)
    }

    fn add_player_guardian(&mut self, source: GuardSource) -> bool {
        if self.player_guard_shield(source).is_none() {
            return false;
        }

        let card = source.take_card(&mut self.player);
        self.message = format!("{} moved to the Guardian Circle.", card.name);
        self.pending_attack.as_mut().unwrap().guardians.push(card);
        self.log(self.message.clone());
        true
    }

    /// Advance one visible battle step. Human trigger choices suspend progression.
    pub fn resolve_pending_attack(&mut self) {
        let Some(mut attack) = self.pending_attack.take() else {
            return;
        };
        if attack.decision.is_some() {
            self.pending_attack = Some(attack);
            return;
        }
        if attack.checked_card.is_some() {
            while !attack.trigger_parts.is_empty() {
                let part = attack.trigger_parts.remove(0);
                let cpu = attack.checking_cpu();
                let (owner, other) = if cpu {
                    (&mut self.cpu, &self.player)
                } else {
                    (&mut self.player, &self.cpu)
                };
                let eligible = trigger_unit_targets(owner, part);
                if part == TriggerPart::Heal {
                    if owner.damage.is_empty() || owner.damage.len() < other.damage.len() {
                        continue;
                    }
                    if !cpu {
                        attack.decision = Some(part);
                        break;
                    }
                    let index = owner.damage.iter().position(|c| !c.face_up).unwrap_or(0);
                    let damage = owner.damage.remove(index);
                    owner.drop_zone.push(damage.card);
                } else if !eligible.is_empty() {
                    if !cpu {
                        attack.decision = Some(part);
                        break;
                    }
                    let preferred = if part == TriggerPart::Stand {
                        eligible
                            .iter()
                            .copied()
                            .find(|u| u.unit(owner).is_some_and(|u| u.rested))
                    } else if attack.step == BattleStep::Drive {
                        Some(attack.attacker)
                    } else {
                        Some(AttackUnit::Vanguard)
                    };
                    let target = preferred
                        .filter(|u| eligible.contains(u))
                        .unwrap_or(eligible[0]);
                    apply_trigger_part(owner, target, part);
                }
            }
            if attack.decision.is_none() {
                let card = attack.checked_card.take().unwrap();
                let owner = if attack.checking_cpu() {
                    &mut self.cpu
                } else {
                    &mut self.player
                };
                if attack.step == BattleStep::Drive {
                    owner.hand.push(card);
                } else {
                    owner.damage.push(DamageCard {
                        card,
                        face_up: true,
                    });
                    attack.damage_dealt += 1;
                }
                self.pending_attack = Some(attack);
                self.refresh_battle_power();
                self.capture_outcome();
            } else {
                self.message = format!(
                    "{}: {}",
                    attack.checked_card.as_ref().unwrap().name,
                    attack.decision.unwrap().instruction()
                );
                self.pending_attack = Some(attack);
                self.refresh_battle_power();
            }
            return;
        }
        match attack.step {
            BattleStep::Guard => {
                attack.step = BattleStep::Drive;
                attack.remaining_drive = if attack.attacker == AttackUnit::Vanguard {
                    let owner = if attack.cpu_attacking {
                        &self.cpu
                    } else {
                        &self.player
                    };
                    if owner.grade() >= 3 {
                        2
                    } else {
                        1
                    }
                } else {
                    0
                };
                self.message = "Guard step complete. Drive checks resolve next.".into();
            }
            BattleStep::Drive => {
                if attack.remaining_drive > 0 {
                    attack.remaining_drive -= 1;
                    let (owner, other) = if attack.cpu_attacking {
                        (&mut self.cpu, &self.player)
                    } else {
                        (&mut self.player, &self.cpu)
                    };
                    if let Some(card) = owner.draw() {
                        owner.last_check = Some(format!("Drive: {}", card.name));
                        self.message = format!("Drive check: {}", card.name);
                        prepare_trigger_check(&mut attack, owner, other.damage.len(), card);
                    }
                } else {
                    attack.step = BattleStep::Damage;
                    self.message = "Drive checks complete. Compare battle power.".into();
                }
            }
            BattleStep::Damage => {
                if !attack.damage_started {
                    let (offense, defense) = if attack.cpu_attacking {
                        (&self.cpu, &mut self.player)
                    } else {
                        (&self.player, &mut self.cpu)
                    };
                    let power = attack.attacker.unit(offense).map_or(0, Unit::power)
                        + attack
                            .booster
                            .and_then(|slot| offense.rear[slot].as_ref())
                            .map_or(0, Unit::power);
                    let defense_power =
                        attack.target.unit(defense).map_or(0, Unit::power) + attack.shield();
                    attack.hit = power >= defense_power;
                    attack.damage_started = true;
                    if attack.hit {
                        match attack.target {
                            AttackUnit::Rear(slot) => {
                                if let Some(unit) = defense.rear[slot].take() {
                                    defense.drop_zone.push(unit.card);
                                }
                                attack.step = BattleStep::Close;
                            }
                            AttackUnit::Vanguard => {
                                attack.remaining_damage = attack
                                    .attacker
                                    .unit(offense)
                                    .map_or(1, Unit::critical)
                                    .max(0)
                                    as usize
                            }
                        }
                    } else {
                        attack.step = BattleStep::Close;
                    }
                    self.message = if attack.hit {
                        "Attack hits. Resolve damage checks."
                    } else {
                        "Guard successful. The attack does not hit."
                    }
                    .into();
                } else if attack.remaining_damage > 0 {
                    attack.remaining_damage -= 1;
                    let (owner, other) = if attack.cpu_attacking {
                        (&mut self.player, &self.cpu)
                    } else {
                        (&mut self.cpu, &self.player)
                    };
                    if let Some(card) = owner.draw() {
                        owner.last_check = Some(format!("Damage: {}", card.name));
                        self.message = format!("Damage check: {}", card.name);
                        prepare_trigger_check(&mut attack, owner, other.damage.len(), card);
                    }
                } else {
                    attack.step = BattleStep::Close;
                }
            }
            BattleStep::Close => {
                let defender = if attack.cpu_attacking {
                    &mut self.player
                } else {
                    &mut self.cpu
                };
                defender.drop_zone.extend(attack.guardians);
                self.message = if !attack.hit {
                    "The attack was guarded.".into()
                } else if attack.target != AttackUnit::Vanguard {
                    format!("{} was retired.", attack.target_name)
                } else {
                    format!(
                        "{} received {} damage check(s).",
                        attack.target_name, attack.damage_dealt
                    )
                };
                self.log(self.message.clone());
                self.capture_outcome();
                return;
            }
        }
        let checking = attack.checked_card.is_some();
        self.pending_attack = Some(attack);
        self.refresh_battle_power();
        if !checking {
            self.capture_outcome();
        }
    }

    pub fn trigger_decision(&self) -> Option<(TriggerPart, &'static str)> {
        let attack = self.pending_attack.as_ref()?;
        Some((attack.decision?, attack.checked_card.as_ref()?.name))
    }

    pub fn trigger_targets(&self) -> Vec<AttackUnit> {
        self.trigger_decision().map_or_else(Vec::new, |(part, _)| {
            trigger_unit_targets(&self.player, part)
        })
    }

    pub fn choose_trigger_unit(&mut self, target: AttackUnit) -> bool {
        if !self.trigger_targets().contains(&target) {
            return false;
        }
        let part = self
            .pending_attack
            .as_mut()
            .unwrap()
            .decision
            .take()
            .unwrap();
        apply_trigger_part(&mut self.player, target, part);
        self.log(format!(
            "{}: {}",
            target.unit(&self.player).unwrap().card.name,
            part.instruction()
        ));
        self.refresh_battle_power();
        true
    }

    pub fn choose_trigger_damage(&mut self, index: usize) -> bool {
        if !self
            .trigger_decision()
            .is_some_and(|(part, _)| part == TriggerPart::Heal)
            || index >= self.player.damage.len()
        {
            return false;
        }
        let damage = self.player.damage.remove(index);
        self.log(format!("Healed {}.", damage.card.name));
        self.player.drop_zone.push(damage.card);
        self.pending_attack.as_mut().unwrap().decision = None;
        true
    }

    pub fn battle_totals(&self) -> Option<(i32, i32, i32)> {
        let a = self.pending_attack.as_ref()?;
        let (offense, defense) = if a.cpu_attacking {
            (&self.cpu, &self.player)
        } else {
            (&self.player, &self.cpu)
        };
        Some((
            a.attacker.unit(offense).map_or(0, Unit::power)
                + a.booster
                    .and_then(|i| offense.rear[i].as_ref())
                    .map_or(0, Unit::power),
            a.target.unit(defense).map_or(0, Unit::power),
            a.shield(),
        ))
    }

    fn refresh_battle_power(&mut self) {
        if let Some((attack, defense, _)) = self.battle_totals() {
            let pending = self.pending_attack.as_mut().unwrap();
            pending.attack_power = attack;
            pending.target_power = defense;
        }
    }

    pub fn can_restand(&self) -> bool {
        self.phase == Phase::Battle
            && self.pending_attack.is_none()
            && self.break_ride_restand
            && self.player.hand.len() >= 3
            && self
                .player
                .vanguard
                .as_ref()
                .is_some_and(|unit| unit.rested)
    }

    pub fn use_restand(&mut self) {
        if !self.can_restand() {
            return;
        }
        restand_vanguard(&mut self.player);
        self.break_ride_restand = false;
        self.message = "Dauntless Drive discards three cards and re-stands the Vanguard.".into();
        self.log(self.message.clone());
    }

    pub fn end_turn(&mut self) {
        if !(self.phase == Phase::Battle || (self.phase == Phase::Call && self.turn == 1))
            || self.pending_attack.is_some()
        {
            return;
        }
        clear_turn_bonuses(&mut self.player);
        clear_turn_bonuses(&mut self.cpu);
        unlock_rear_guards(&mut self.player);
        self.turn += 1;
        self.phase = Phase::Cpu;
        self.cpu_attack_index = 0;
        self.cpu_stage = TurnStage::Stand;
        self.message = format!("{} begins the Stand phase.", self.cpu.name);
    }

    pub fn advance_cpu_turn(&mut self) {
        if self.phase != Phase::Cpu || self.pending_attack.is_some() {
            return;
        }
        match self.cpu_stage {
            TurnStage::Stand => {
                ready_player(&mut self.cpu);
                self.cpu_stage = TurnStage::Draw;
            }
            TurnStage::Draw => {
                self.cpu.draw_to_hand();
                self.cpu_stage = TurnStage::Ride;
                self.capture_outcome();
            }
            TurnStage::Ride => {
                cpu_ride(&mut self.cpu);
                self.cpu_stage = TurnStage::Main;
            }
            TurnStage::Main => {
                for _ in 0..2 {
                    cpu_call(&mut self.cpu);
                }
                cpu_limit_break(&mut self.cpu, &mut self.player);
                self.cpu_stage = if self.turn == 1 {
                    TurnStage::End
                } else {
                    TurnStage::Battle
                };
            }
            TurnStage::Battle => self.advance_cpu_battle(),
            TurnStage::End => {
                clear_turn_bonuses(&mut self.player);
                clear_turn_bonuses(&mut self.cpu);
                unlock_rear_guards(&mut self.cpu);
                self.log(format!("{} completed its turn.", self.cpu.name));
                self.start_player_turn();
            }
        }
        if self.phase == Phase::Cpu && self.pending_attack.is_none() {
            self.message = format!("{} · {} phase", self.cpu.name, self.cpu_stage.label());
        }
    }

    fn advance_cpu_battle(&mut self) {
        while let Some(&attacker) = AttackUnit::CPU_ATTACK_ORDER.get(self.cpu_attack_index) {
            self.cpu_attack_index += 1;
            if self.declare_cpu_attack(attacker) {
                return;
            }
        }

        if self.cpu_attack_index == AttackUnit::CPU_ATTACK_ORDER.len() {
            self.cpu_attack_index += 1;
            if self.cpu.hand.len() >= 3
                && self.cpu.vanguard.as_ref().is_some_and(|unit| {
                    unit.rested && unit.power_bonus >= 10000 && unit.card.clan == "Kagero"
                })
            {
                restand_vanguard(&mut self.cpu);
                if self.declare_cpu_attack(AttackUnit::Vanguard) {
                    return;
                }
            }
        }

        self.cpu_stage = TurnStage::End;
    }

    fn declare_cpu_attack(&mut self, attacker: AttackUnit) -> bool {
        if !attacker.can_attack(&self.cpu) {
            return false;
        }
        let booster = attacker.booster(&self.cpu);
        let power = attacker.unit(&self.cpu).unwrap().power() + booster.map_or(0, Unit::power);
        let vanguard = AttackUnit::Vanguard;
        let target = if power < self.player.vanguard_power() {
            [AttackUnit::Rear(1), AttackUnit::Rear(3)]
                .into_iter()
                .find(|&target| {
                    target.can_be_targeted(&self.player)
                        && target.unit(&self.player).unwrap().power() <= power
                })
                .unwrap_or(vanguard)
        } else {
            vanguard
        };
        self.open_attack(true, attacker, target, booster.is_some())
    }

    fn open_attack(
        &mut self,
        cpu_attacking: bool,
        attacker: AttackUnit,
        target: AttackUnit,
        use_boost: bool,
    ) -> bool {
        let (offense, defense) = if cpu_attacking {
            (&mut self.cpu, &mut self.player)
        } else {
            (&mut self.player, &mut self.cpu)
        };
        if !attacker.can_attack(offense) || !target.can_be_targeted(defense) {
            return false;
        }
        let booster = if use_boost {
            if attacker.booster(offense).is_none() {
                return false;
            }
            attacker.boost_slot()
        } else {
            None
        };
        let boost_power = booster.map_or(0, |slot| {
            let unit = offense.rear[slot].as_mut().unwrap();
            unit.rested = true;
            unit.power()
        });
        let attacking_unit = attacker.unit_mut(offense).unwrap();
        attacking_unit.rested = true;
        let target_unit = target.unit(defense).unwrap();
        let target_name = target_unit.card.name;
        let attack_power = attacking_unit.power() + boost_power;
        let target_power = target_unit.power();
        let guardians = if cpu_attacking {
            Vec::new()
        } else {
            auto_guard(defense, target, attack_power - target_power + 1)
        };
        self.pending_attack = Some(PendingAttack {
            cpu_attacking,
            attacker,
            target,
            booster,
            attack_power,
            target_power,
            guardians,
            step: BattleStep::Guard,
            checked_card: None,
            decision: None,
            trigger_parts: Vec::new(),
            remaining_drive: 0,
            remaining_damage: 0,
            damage_started: false,
            damage_dealt: 0,
            hit: false,
            target_name,
        });
        self.message = format!(
            "{} attacks {target_name} for {attack_power} power.",
            attacking_unit.card.name,
        );
        self.log(self.message.clone());
        true
    }

    fn start_player_turn(&mut self) {
        self.turn += 1;
        self.player_calls = 0;
        self.break_ride_restand = false;
        self.phase = Phase::Stand;
        self.log(format!("Turn {} — your Stand phase.", self.turn));
    }

    pub fn advance_player_turn(&mut self) {
        match self.phase {
            Phase::Stand => {
                ready_player(&mut self.player);
                self.phase = Phase::Draw;
            }
            Phase::Draw => {
                self.player.draw_to_hand();
                self.phase = Phase::Ride;
                self.log(format!("Turn {} — drew one card.", self.turn));
                self.capture_outcome();
            }
            _ => {}
        }
    }

    fn capture_outcome(&mut self) {
        if self.cpu.damage.len() >= 6 || self.cpu.deck_out {
            self.winner = Some(true);
            self.phase = Phase::Finished;
            self.message = "Victory! The cardfight is yours.".into();
        } else if self.player.damage.len() >= 6 || self.player.deck_out {
            self.winner = Some(false);
            self.phase = Phase::Finished;
            self.message = "Defeat. Regroup and try the fight again.".into();
        }
        if self.winner.is_some() {
            if let Some(attack) = self.pending_attack.take() {
                let defender = if attack.cpu_attacking {
                    &mut self.player
                } else {
                    &mut self.cpu
                };
                defender.drop_zone.extend(attack.guardians);
            }
        }
    }
}

fn pay_counter_blast(player: &mut Player, count: usize) {
    let mut paid = 0;
    for damage in &mut player.damage {
        if damage.face_up && paid < count {
            damage.face_up = false;
            paid += 1;
        }
    }
}

fn clear_turn_bonuses(player: &mut Player) {
    for unit in player
        .vanguard
        .iter_mut()
        .chain(player.rear.iter_mut().flatten())
    {
        unit.power_bonus = 0;
        unit.critical_bonus = 0;
    }
}

fn trigger_unit_targets(player: &Player, part: TriggerPart) -> Vec<AttackUnit> {
    if part == TriggerPart::Heal {
        return Vec::new();
    }
    std::iter::once(AttackUnit::Vanguard)
        .chain((0..5).map(AttackUnit::Rear))
        .filter(|unit| part != TriggerPart::Stand || *unit != AttackUnit::Vanguard)
        .filter(|unit| unit.unit(player).is_some_and(|u| !u.locked))
        .collect()
}

fn apply_trigger_part(player: &mut Player, target: AttackUnit, part: TriggerPart) {
    if let Some(unit) = target.unit_mut(player) {
        match part {
            TriggerPart::Power => unit.power_bonus += 5000,
            TriggerPart::Critical => unit.critical_bonus += 1,
            TriggerPart::Stand => unit.rested = false,
            TriggerPart::Heal => {}
        }
    }
}

fn prepare_trigger_check(
    attack: &mut PendingAttack,
    owner: &mut Player,
    opponent_damage: usize,
    card: Card,
) {
    attack.trigger_parts.clear();
    if card.is_trigger() && matches_clan(owner, &card) {
        attack.trigger_parts.push(TriggerPart::Power);
        match card.trigger {
            Trigger::Critical => attack.trigger_parts.push(TriggerPart::Critical),
            Trigger::Stand => attack.trigger_parts.push(TriggerPart::Stand),
            Trigger::Heal if !owner.damage.is_empty() && owner.damage.len() >= opponent_damage => {
                attack.trigger_parts.push(TriggerPart::Heal)
            }
            Trigger::Draw => owner.draw_to_hand(),
            _ => {}
        }
    }
    attack.checked_card = Some(card);
}

fn ready_player(player: &mut Player) {
    clear_turn_bonuses(player);
    for unit in player.rear.iter_mut().flatten() {
        unit.rested = false;
        unit.power_bonus = 0;
    }
    if let Some(vanguard) = player.vanguard.as_mut() {
        vanguard.rested = false;
        vanguard.power_bonus = 0;
    }
}

fn unlock_rear_guards(player: &mut Player) {
    for unit in player.rear.iter_mut().flatten() {
        unit.locked = false;
    }
}

fn cpu_ride(player: &mut Player) {
    let grade = player.grade();
    let best = player
        .hand
        .iter()
        .enumerate()
        .filter(|(_, card)| {
            !card.is_trigger() && card.grade >= grade && card.grade <= grade + 1 && card.grade <= 3
        })
        .max_by_key(|(_, card)| card.grade)
        .map(|(index, _)| index);
    if let Some(index) = best {
        let card = player.hand.remove(index);
        let previous = player.vanguard.replace(Unit::new(card)).unwrap();
        player.soul.push(previous.card);
    }
}

fn cpu_call(player: &mut Player) {
    let Some(slot) = player.rear.iter().position(Option::is_none) else {
        return;
    };
    let grade = player.grade();
    let choice = player
        .hand
        .iter()
        .enumerate()
        .filter(|(_, card)| !card.is_trigger() && card.grade <= grade)
        .max_by_key(|(_, card)| card.power)
        .map(|(index, _)| index);
    if let Some(index) = choice {
        let card = player.hand.remove(index);
        player.rear[slot] = Some(Unit::new(card));
    }
}

fn cpu_limit_break(cpu: &mut Player, player: &mut Player) {
    let active = cpu
        .vanguard
        .as_ref()
        .map_or(false, |unit| unit.card.limit_break)
        && cpu.damage.len() >= 4
        && cpu.face_up_damage() >= 2;
    if !active {
        return;
    }
    pay_counter_blast(cpu, 2);
    if let Some(vanguard) = cpu.vanguard.as_mut() {
        vanguard.power_bonus += 10000;
    }
    if cpu
        .vanguard
        .as_ref()
        .map_or(false, |unit| unit.card.clan == "Link Joker")
    {
        if let Some(slot) = [0usize, 2, 4]
            .into_iter()
            .find(|slot| player.rear[*slot].is_some())
        {
            if let Some(unit) = player.rear[slot].as_mut() {
                unit.locked = true;
            }
        }
    }
}

fn restand_vanguard(player: &mut Player) {
    let discard_start = player.hand.len() - 3;
    player
        .drop_zone
        .extend(player.hand.drain(discard_start..).rev());
    player.vanguard.as_mut().unwrap().rested = false;
}

#[derive(Clone, Copy)]
enum GuardSource {
    Hand(usize),
    Intercept(usize),
}

impl GuardSource {
    fn shield(self, defender: &Player, target: AttackUnit) -> Option<i32> {
        let card = match self {
            Self::Hand(index) => defender
                .hand
                .get(index)
                .filter(|card| card.grade <= defender.grade())?,
            Self::Intercept(slot) => {
                let interceptor = AttackUnit::Rear(slot);
                if interceptor == target || !interceptor.can_be_targeted(defender) {
                    return None;
                }
                &interceptor
                    .unit(defender)
                    .filter(|unit| unit.card.grade == 2)?
                    .card
            }
        };
        (card.shield > 0).then_some(card.shield)
    }

    fn take_card(self, defender: &mut Player) -> Card {
        match self {
            Self::Hand(index) => defender.hand.remove(index),
            Self::Intercept(slot) => defender.rear[slot].take().unwrap().card,
        }
    }
}

fn auto_guard(defender: &mut Player, target: AttackUnit, needed: i32) -> Vec<Card> {
    if needed <= 0 {
        return Vec::new();
    }

    let mut guards: Vec<_> = (0..defender.hand.len())
        .map(GuardSource::Hand)
        .chain([1, 3].into_iter().map(GuardSource::Intercept))
        .filter_map(|source| {
            source
                .shield(defender, target)
                .map(|shield| (source, shield))
        })
        .collect();
    if guards.iter().map(|(_, shield)| shield).sum::<i32>() < needed {
        return Vec::new();
    }

    guards.sort_by_key(|(_, shield)| *shield);
    let mut chosen = Vec::new();
    let mut total = 0;
    for (source, shield) in guards {
        chosen.push(source);
        total += shield;
        if total >= needed {
            break;
        }
    }
    // Removing hand cards from right to left preserves the remaining chosen indices.
    chosen.sort_by_key(|source| {
        std::cmp::Reverse(match source {
            GuardSource::Hand(index) | GuardSource::Intercept(index) => *index,
        })
    });
    chosen
        .into_iter()
        .map(|source| source.take_card(defender))
        .collect()
}

fn matches_clan(player: &Player, card: &Card) -> bool {
    player
        .vanguard
        .as_ref()
        .is_some_and(|unit| unit.card.clan == card.clan)
        || player
            .rear
            .iter()
            .flatten()
            .any(|unit| !unit.locked && unit.card.clan == card.clan)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::story::CHAPTERS;

    fn card(name: &'static str, grade: u8, power: i32, shield: i32) -> Card {
        Card::unit(name, "Royal Paladin", grade, power, shield)
    }

    fn unit(name: &'static str, grade: u8, power: i32, shield: i32) -> Option<Unit> {
        Some(Unit::new(card(name, grade, power, shield)))
    }

    fn battle() -> MatchGame {
        let mut game = MatchGame::new(CHAPTERS[0]);
        let deck = vec![Card::trigger("Deck card", "Other Clan", Trigger::Stand); 10];
        game.player = Player::new("You", deck.clone());
        game.cpu = Player::new("Opponent", deck);
        game.player.vanguard = unit("Your Vanguard", 2, 10000, 5000);
        game.cpu.vanguard = unit("Opposing Vanguard", 2, 10000, 5000);
        game.phase = Phase::Battle;
        game.turn = 2;
        game
    }

    fn finish_attack(game: &mut MatchGame) {
        for _ in 0..80 {
            if game.pending_attack.is_none() {
                return;
            }
            if let Some((part, _)) = game.trigger_decision() {
                if part == TriggerPart::Heal {
                    assert!(game.choose_trigger_damage(0));
                } else {
                    let target = if part == TriggerPart::Stand {
                        AttackUnit::Rear(1)
                    } else {
                        AttackUnit::Vanguard
                    };
                    assert!(game.choose_trigger_unit(target));
                }
            } else {
                game.resolve_pending_attack();
            }
        }
        panic!("attack failed to complete");
    }

    #[test]
    fn flipping_damage_updates_costs_without_moving_or_reordering_cards() {
        let mut game = battle();
        game.player.vanguard.as_mut().unwrap().card.limit_break = true;
        game.player.damage = (0..4)
            .map(|_| DamageCard {
                card: card("Damage", 1, 6000, 5000),
                face_up: true,
            })
            .collect();
        assert!(game.can_limit_break());
        assert!(game.flip_damage(2));
        assert!(!game.player.damage[2].face_up);
        assert_eq!(game.player.face_up_damage(), 3);
        game.use_limit_break();
        assert_eq!(game.player.face_up_damage(), 1);
        assert!(!game.can_limit_break());
        assert_eq!(game.player.damage.len(), 4);
        assert!(game.flip_damage(2));
        assert_eq!(game.player.face_up_damage(), 2);
        assert!(game.can_limit_break());
        assert_eq!(game.player.damage[2].card.name, "Damage");
        assert!(!game.flip_damage(4));
        assert!(!game.flip_damage(usize::MAX));
        assert!(game.cpu.damage.is_empty());
    }

    #[test]
    fn damage_cannot_be_manually_changed_during_opponent_turn_battle_or_result() {
        let mut game = battle();
        game.player.damage.push(DamageCard {
            card: card("Damage", 1, 6000, 5000),
            face_up: true,
        });
        assert!(game.declare_attack(AttackUnit::Vanguard, AttackUnit::Vanguard, false));
        assert!(!game.flip_damage(0));
        game.pending_attack = None;
        for phase in [Phase::Cpu, Phase::Finished] {
            game.phase = phase;
            assert!(!game.flip_damage(0));
            assert!(game.player.damage[0].face_up);
        }
        game.phase = Phase::Call;
        game.winner = Some(true);
        assert!(!game.flip_damage(0));
    }

    #[test]
    fn attack_targets_only_include_occupied_unlocked_front_row_units() {
        let mut game = battle();
        game.player.rear[0] = unit("Unused booster", 1, 6000, 5000);
        game.player.rear[1] = unit("Attacking rear-guard", 2, 10000, 5000);
        game.cpu.rear[1] = unit("Locked opposing rear-guard", 2, 10000, 5000);
        game.cpu.rear[1].as_mut().unwrap().locked = true;
        game.cpu.rear[3] = unit("Rested opposing rear-guard", 2, 10000, 5000);
        game.cpu.rear[3].as_mut().unwrap().rested = true;
        game.cpu.rear[4] = unit("Opposing booster", 1, 6000, 5000);

        let attacker = AttackUnit::Rear(1);
        assert_eq!(
            game.attack_targets(),
            [AttackUnit::Vanguard, AttackUnit::Rear(3)]
        );
        assert!(!game.declare_attack(attacker, AttackUnit::Rear(1), true));
        assert!(!game.declare_attack(attacker, AttackUnit::Rear(4), true));
        assert!(game.can_attack(attacker));
        assert!(game.declare_attack(attacker, AttackUnit::Rear(3), false));
        assert!(!game.player.rear[0].as_ref().unwrap().rested);

        finish_attack(&mut game);
        assert!(game.cpu.rear[3].is_none());
        assert!(game.cpu.rear[4].is_some());
        assert_eq!(game.cpu.drop_zone[0].name, "Rested opposing rear-guard");
        assert!(game.cpu.damage.is_empty());
        assert!(!game.can_attack(attacker));
    }

    #[test]
    fn each_column_uses_its_own_optional_booster() {
        let mut game = battle();
        for slot in [1, 3] {
            game.player.rear[slot] = unit("Attacking rear-guard", 2, 9000, 5000);
        }
        for slot in [0, 2, 4] {
            game.player.rear[slot] = unit("Booster", 0, 5000, 5000);
        }
        let left_booster = game.player.rear[0].as_mut().unwrap();
        left_booster.power_bonus = 3000;
        left_booster.locked = true;
        assert!(!game.declare_attack(AttackUnit::Rear(1), AttackUnit::Vanguard, true));
        game.player.rear[0].as_mut().unwrap().locked = false;

        for (attacker, slot, expected_power) in [
            (AttackUnit::Rear(1), 0, 17000),
            (AttackUnit::Vanguard, 2, 15000),
            (AttackUnit::Rear(3), 4, 14000),
        ] {
            assert!(game.declare_attack(attacker, AttackUnit::Vanguard, true));
            let attack = game.pending_attack.as_ref().unwrap();
            assert_eq!(attack.booster, Some(slot));
            assert_eq!(attack.attack_power, expected_power);
            assert!(game.player.rear[slot].as_ref().unwrap().rested);
            assert!(game.boost_for(attacker).is_none());
            finish_attack(&mut game);
        }
        assert_eq!(game.cpu.damage.len(), 3);
    }

    #[test]
    fn opponent_stages_advance_their_actions_once_and_wait_for_guarding() {
        let mut game = battle();
        game.cpu.vanguard.as_mut().unwrap().rested = true;
        game.cpu.hand = vec![card("Next ride", 3, 11000, 0), card("Call", 2, 9000, 5000)];
        let deck_count = game.cpu.deck.len();
        game.end_turn();
        assert_eq!(game.stage(), Some(TurnStage::Stand));
        assert!(game.cpu.vanguard.as_ref().unwrap().rested);
        game.advance_cpu_turn();
        assert!(!game.cpu.vanguard.as_ref().unwrap().rested);
        assert_eq!(game.stage(), Some(TurnStage::Draw));
        assert_eq!(game.cpu.deck.len(), deck_count);
        game.advance_cpu_turn();
        assert_eq!(game.stage(), Some(TurnStage::Ride));
        assert_eq!(game.cpu.deck.len(), deck_count - 1);
        game.advance_cpu_turn();
        assert_eq!(game.stage(), Some(TurnStage::Main));
        assert_eq!(game.cpu.grade(), 3);
        assert!(game.cpu.rear.iter().all(Option::is_none));
        game.advance_cpu_turn();
        assert_eq!(game.stage(), Some(TurnStage::Battle));
        assert!(game.cpu.rear.iter().any(Option::is_some));
        game.advance_cpu_turn();
        let attacker = game.pending_attack.as_ref().unwrap().attacker;
        let hand_count = game.cpu.hand.len();
        for _ in 0..10 {
            game.advance_cpu_turn();
        }
        assert_eq!(game.pending_attack.as_ref().unwrap().attacker, attacker);
        assert_eq!(game.cpu.hand.len(), hand_count);
        assert!(game.player.damage.is_empty());
    }

    #[test]
    fn each_cpu_attack_pauses_for_a_guard_decision() {
        let mut game = battle();
        game.cpu.rear[1] = unit("First attacker", 2, 10000, 5000);
        game.cpu.rear[3] = unit("Second attacker", 2, 13000, 5000);
        game.cpu.rear[4] = unit("CPU booster", 1, 6000, 5000);
        game.player.hand = vec![
            card("First guardian", 1, 6000, 5000),
            card("Second guardian", 2, 9000, 5000),
            card("Over-grade guardian", 3, 10000, 5000),
        ];
        game.end_turn();
        for _ in 0..5 {
            game.advance_cpu_turn();
        }
        assert_eq!(game.phase, Phase::Cpu);
        assert_eq!(
            game.pending_attack.as_ref().unwrap().attacker,
            AttackUnit::Rear(1)
        );
        assert!(game.player.damage.is_empty());

        finish_attack(&mut game);
        assert_eq!(game.player.damage.len(), 1);
        assert_eq!(game.player.hand.len(), 3);
        game.advance_cpu_turn();
        assert_eq!(game.pending_attack.as_ref().unwrap().attack_power, 19000);
        assert!(!game.call_guardian(2));
        assert!(game.call_guardian(0));
        assert!(game.call_guardian(0));
        assert_eq!(game.player.hand[0].name, "Over-grade guardian");
        assert_eq!(game.pending_attack.as_ref().unwrap().shield(), 10000);
        assert!(game.player.drop_zone.is_empty());

        finish_attack(&mut game);
        assert_eq!(game.player.damage.len(), 1);
        assert_eq!(game.player.drop_zone.len(), 2);
        assert!(game.pending_attack.is_none());
        game.advance_cpu_turn();
        assert_eq!(
            game.pending_attack.as_ref().unwrap().attacker,
            AttackUnit::Vanguard
        );
        finish_attack(&mut game);
        game.advance_cpu_turn();
        assert_eq!(game.stage(), Some(TurnStage::End));
        game.advance_cpu_turn();
        assert_eq!(game.phase, Phase::Stand);
        assert_eq!(game.turn, 4);
        game.advance_player_turn();
        game.advance_player_turn();
        assert_eq!(game.phase, Phase::Ride);
    }

    #[test]
    fn rested_front_rear_guard_can_intercept_for_another_unit() {
        let mut game = battle();
        game.cpu.rear[1] = unit("Attacking rear-guard", 2, 9000, 5000);
        game.player.rear[1] = unit("Targeted rear-guard", 2, 8000, 5000);
        game.player.rear[3] = unit("Intercepting rear-guard", 2, 9000, 5000);
        game.player.rear[3].as_mut().unwrap().rested = true;
        game.end_turn();
        for _ in 0..5 {
            game.advance_cpu_turn();
        }
        assert_eq!(
            game.pending_attack.as_ref().unwrap().target,
            AttackUnit::Rear(1)
        );
        assert!(!game.can_intercept(1));
        assert!(game.intercept(3));
        assert!(game.player.rear[3].is_none());
        assert_eq!(game.pending_attack.as_ref().unwrap().shield(), 5000);
        assert!(game.player.drop_zone.is_empty());
        finish_attack(&mut game);
        assert!(game.player.rear[1].is_some());
        assert_eq!(game.player.drop_zone[0].name, "Intercepting rear-guard");
        assert!(game.player.damage.is_empty());
    }

    #[test]
    fn drive_check_can_punch_through_a_guard_for_a_rear_guard_target() {
        let mut game = battle();
        game.player.deck.push(Card::trigger(
            "Drive trigger",
            "Royal Paladin",
            Trigger::Stand,
        ));
        game.cpu.rear[1] = unit("Targeted rear-guard", 2, 10000, 5000);
        game.cpu.hand.push(card("CPU guardian", 1, 6000, 5000));
        assert!(game.declare_attack(AttackUnit::Vanguard, AttackUnit::Rear(1), false));
        assert!(game.cpu.hand.is_empty());
        assert_eq!(game.pending_attack.as_ref().unwrap().shield(), 5000);
        assert!(game.cpu.drop_zone.is_empty());
        assert!(game.player.hand.is_empty());

        finish_attack(&mut game);
        assert_eq!(game.player.hand[0].name, "Drive trigger");
        assert!(game.cpu.rear[1].is_none());
        assert_eq!(game.cpu.drop_zone.len(), 2);
        assert!(game.cpu.damage.is_empty());
    }

    #[test]
    fn stand_allows_another_rear_guard_attack_but_critical_does_not() {
        for kind in [Trigger::Stand, Trigger::Critical] {
            let mut game = battle();
            let rear_guard = AttackUnit::Rear(1);
            game.player.rear[1] = unit("Attacking rear-guard", 2, 9000, 5000);
            game.cpu.rear[1] = unit("Targeted rear-guard", 2, 9000, 5000);
            assert!(game.declare_attack(rear_guard, AttackUnit::Rear(1), false));
            finish_attack(&mut game);
            assert!(!game.can_attack(rear_guard));

            game.player
                .deck
                .push(Card::trigger("Drive trigger", "Royal Paladin", kind));
            assert!(game.declare_attack(AttackUnit::Vanguard, AttackUnit::Vanguard, false));
            finish_attack(&mut game);

            assert_eq!(game.can_attack(rear_guard), kind == Trigger::Stand);
            assert_eq!(
                game.cpu.damage.len(),
                if kind == Trigger::Critical { 2 } else { 1 }
            );
        }
    }
    #[test]
    fn opening_draw_and_first_turn_skip_battle() {
        let mut game = MatchGame::new(CHAPTERS[0]);
        assert_eq!(game.player.hand.len(), 5);
        assert_eq!(game.player.deck.len(), 44);
        game.play_rps(RpsChoice::Paper, RpsChoice::Rock);
        game.finish_rps_reveal();
        game.choose_turn_order(true);
        game.mulligan(&[]);
        game.confirm_opening_hands();
        game.begin_fight();
        assert_eq!(game.phase, Phase::Stand);
        game.advance_player_turn();
        assert_eq!(game.player.hand.len(), 5);
        game.advance_player_turn();
        assert_eq!(game.player.hand.len(), 6);
        assert_eq!(game.player.deck.len(), 43);
        game.skip_ride();
        game.begin_battle();
        assert_eq!(game.stage(), Some(TurnStage::Main));
        assert!(!game.can_attack(AttackUnit::Vanguard));
        game.end_turn();
        assert_eq!(game.phase, Phase::Cpu);
        assert_eq!(game.turn, 2);
    }

    #[test]
    fn normal_calls_include_triggers_and_retire_overcalled_units() {
        let mut game = battle();
        game.phase = Phase::Call;
        game.player.hand = vec![
            Card::trigger("Stand", "Royal Paladin", Trigger::Stand),
            card("Invalid ride", 0, 5000, 10000),
        ];
        game.player.rear[0] = unit("Old unit", 1, 7000, 5000);
        assert!(game.call_options().contains(&0));
        game.call(0, 0);
        assert_eq!(game.player.rear[0].as_ref().unwrap().card.name, "Stand");
        assert_eq!(game.player.drop_zone[0].name, "Old unit");
        game.phase = Phase::Ride;
        game.ride(0);
        assert_eq!(game.player.hand.len(), 1);
        assert_eq!(game.player.grade(), 2);
    }

    #[test]
    fn split_critical_power_updates_boost_total_and_expires_at_end() {
        let mut game = battle();
        game.player.rear[2] = unit("Booster", 1, 7000, 5000);
        game.player.deck.push(Card::trigger(
            "Critical",
            "Royal Paladin",
            Trigger::Critical,
        ));
        assert!(game.declare_attack(AttackUnit::Vanguard, AttackUnit::Vanguard, true));
        game.resolve_pending_attack();
        game.resolve_pending_attack();
        game.resolve_pending_attack();
        assert_eq!(game.trigger_decision().unwrap().0, TriggerPart::Power);
        game.resolve_pending_attack(); // A pending decision cannot be bypassed.
        assert_eq!(game.trigger_decision().unwrap().0, TriggerPart::Power);
        assert!(game.choose_trigger_unit(AttackUnit::Rear(2)));
        assert_eq!(game.battle_totals().unwrap().0, 22000);
        game.resolve_pending_attack();
        assert_eq!(game.trigger_decision().unwrap().0, TriggerPart::Critical);
        assert!(game.choose_trigger_unit(AttackUnit::Vanguard));
        finish_attack(&mut game);
        assert_eq!(game.cpu.damage.len(), 2);
        assert_eq!(game.player.vanguard.as_ref().unwrap().critical(), 2);
        game.end_turn();
        assert_eq!(game.player.vanguard.as_ref().unwrap().critical(), 1);
        assert_eq!(game.player.rear[2].as_ref().unwrap().power(), 7000);
    }

    #[test]
    fn sixth_damage_heal_pauses_cpu_and_can_remove_face_down_damage() {
        let mut game = battle();
        game.phase = Phase::Cpu;
        for player in [&mut game.player, &mut game.cpu] {
            player.damage = (0..5)
                .map(|_| DamageCard {
                    card: card("Damage", 1, 6000, 5000),
                    face_up: false,
                })
                .collect();
        }
        game.player
            .deck
            .push(Card::trigger("Heal", "Royal Paladin", Trigger::Heal));
        game.cpu.rear[1] = unit("Attacker", 2, 10000, 5000);
        assert!(game.open_attack(true, AttackUnit::Rear(1), AttackUnit::Vanguard, false));
        for _ in 0..20 {
            if game.trigger_decision().is_some() {
                break;
            }
            game.resolve_pending_attack();
        }
        assert_eq!(game.trigger_decision().unwrap().0, TriggerPart::Power);
        assert_eq!(game.player.damage.len(), 5);
        assert!(game.choose_trigger_unit(AttackUnit::Vanguard));
        game.resolve_pending_attack();
        assert_eq!(game.trigger_decision().unwrap().0, TriggerPart::Heal);
        game.advance_cpu_turn();
        assert_eq!(game.trigger_decision().unwrap().0, TriggerPart::Heal);
        assert!(game.choose_trigger_damage(3));
        finish_attack(&mut game);
        assert_eq!(game.player.damage.len(), 5);
        assert!(game.player.damage.last().unwrap().face_up);
        assert_eq!(game.player.drop_zone[0].name, "Damage");
        assert_eq!(game.winner, None);
    }

    #[test]
    fn draw_trigger_adds_one_card_and_stand_cannot_target_vanguard() {
        let mut game = battle();
        game.player
            .deck
            .push(Card::trigger("Draw", "Royal Paladin", Trigger::Draw));
        assert!(game.declare_attack(AttackUnit::Vanguard, AttackUnit::Vanguard, false));
        finish_attack(&mut game);
        assert_eq!(game.player.hand.len(), 2);
        assert_eq!(
            Card::trigger("Draw", "Royal Paladin", Trigger::Draw).shield,
            5000
        );
        assert!(trigger_unit_targets(&game.player, TriggerPart::Stand).is_empty());
    }

    #[test]
    fn damage_power_does_not_cancel_remaining_critical_damage() {
        let mut game = battle();
        game.player.vanguard.as_mut().unwrap().critical_bonus = 1;
        game.cpu
            .deck
            .push(Card::trigger("Defense", "Royal Paladin", Trigger::Stand));
        assert!(game.declare_attack(AttackUnit::Vanguard, AttackUnit::Vanguard, false));
        finish_attack(&mut game);
        assert_eq!(game.cpu.damage.len(), 2);
        assert_eq!(game.cpu.vanguard_power(), 15000);
    }

    #[test]
    fn empty_deck_loses_after_the_last_draw_resolves() {
        let mut game = battle();
        game.player.deck = vec![card("Last card", 1, 7000, 5000)];
        assert!(game.declare_attack(AttackUnit::Vanguard, AttackUnit::Vanguard, false));
        finish_attack(&mut game);
        assert_eq!(game.player.hand.len(), 1);
        assert_eq!(game.winner, Some(false));
        assert_eq!(game.phase, Phase::Finished);
    }
    #[test]
    fn rps_ties_replay_and_only_the_winner_chooses_order() {
        for human in RpsChoice::ALL {
            for cpu in RpsChoice::ALL {
                let mut game = MatchGame::new(CHAPTERS[0]);
                assert!(!game.choose_turn_order(false));
                assert!(game.play_rps(human, cpu));
                assert!(!game.play_rps(human, cpu));
                game.finish_rps_reveal();
                if human == cpu {
                    assert_eq!(game.setup_stage, SetupStage::RockPaperScissors);
                } else if human.beats(cpu) {
                    assert_eq!(game.setup_stage, SetupStage::ChooseOrder);
                    assert!(game.choose_turn_order(false));
                    assert!(!game.player_first);
                } else {
                    assert_eq!(game.setup_stage, SetupStage::Mulligan);
                    assert!(!game.choose_turn_order(true));
                    assert!(!game.player_first);
                }
                assert_eq!(game.player.hand.len(), 5);
            }
        }
    }

    #[test]
    fn mulligan_is_once_only_and_draws_before_returned_cards_are_shuffled() {
        let mut game = MatchGame::new(CHAPTERS[0]);
        game.play_rps(RpsChoice::Paper, RpsChoice::Rock);
        game.finish_rps_reveal();
        game.choose_turn_order(true);
        game.player.hand = vec![card("Return", 0, 5000, 10000); 5];
        game.player.deck = vec![card("Replacement", 1, 7000, 5000); 12];
        assert!(!game.mulligan(&[0, 0]));
        assert!(!game.mulligan(&[5]));
        assert_eq!(game.player.hand.len(), 5);
        assert!(game.mulligan(&[0, 1, 2, 3, 4]));
        assert!(game.player.hand.iter().all(|c| c.name == "Replacement"));
        assert_eq!(game.player.deck.len(), 12);
        assert_eq!(
            game.player
                .deck
                .iter()
                .filter(|c| c.name == "Return")
                .count(),
            5
        );
        assert!(!game.mulligan(&[]));
        assert_eq!(game.setup_stage, SetupStage::HandsReady);
        assert!(game.confirm_opening_hands());
        assert!(!game.confirm_opening_hands());
        assert_eq!(game.setup_stage, SetupStage::StandUp);
        assert_eq!(game.phase, Phase::Setup);
    }

    #[test]
    fn cpu_going_first_skips_battle_and_human_second_can_attack() {
        let mut game = MatchGame::new(CHAPTERS[0]);
        game.play_rps(RpsChoice::Rock, RpsChoice::Paper);
        game.finish_rps_reveal();
        assert!(game.mulligan(&[]));
        game.confirm_opening_hands();
        game.begin_fight();
        assert_eq!(game.phase, Phase::Cpu);
        assert_eq!(game.turn, 1);
        for _ in 0..4 {
            game.advance_cpu_turn();
        }
        assert_eq!(game.stage(), Some(TurnStage::End));
        assert!(game.pending_attack.is_none());
        assert!(game.player.damage.is_empty());
        game.advance_cpu_turn();
        assert_eq!(game.phase, Phase::Stand);
        assert_eq!(game.turn, 2);
        game.advance_player_turn();
        game.advance_player_turn();
        game.skip_ride();
        game.begin_battle();
        assert!(game.can_attack(AttackUnit::Vanguard));
    }

    #[test]
    fn human_stand_precedes_draw_and_cannot_draw_twice() {
        let mut game = battle();
        game.phase = Phase::Stand;
        game.player.rear[0] = unit("Rested booster", 1, 7000, 5000);
        game.player.rear[0].as_mut().unwrap().rested = true;
        let hand = game.player.hand.len();
        let deck = game.player.deck.len();
        assert!(game.player.rear[0].as_ref().unwrap().rested);
        game.advance_player_turn();
        assert!(!game.player.rear[0].as_ref().unwrap().rested);
        assert_eq!(game.phase, Phase::Draw);
        assert_eq!(game.player.hand.len(), hand);
        game.advance_player_turn();
        assert_eq!(game.phase, Phase::Ride);
        assert_eq!(game.player.hand.len(), hand + 1);
        assert_eq!(game.player.deck.len(), deck - 1);
        game.advance_player_turn();
        assert_eq!(game.player.hand.len(), hand + 1);
    }
    #[test]
    fn locked_rear_guard_unlocks_at_its_owners_end_step() {
        let mut game = battle();
        game.player.rear[0] = unit("Locked", 1, 7000, 5000);
        game.player.rear[0].as_mut().unwrap().locked = true;
        game.phase = Phase::Cpu;
        game.cpu_stage = TurnStage::End;
        game.advance_cpu_turn();
        assert!(game.player.rear[0].as_ref().unwrap().locked);
        game.advance_player_turn();
        game.advance_player_turn();
        game.skip_ride();
        game.begin_battle();
        game.end_turn();
        assert!(!game.player.rear[0].as_ref().unwrap().locked);
    }
}
