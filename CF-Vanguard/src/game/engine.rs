//! Turn state and cardfight actions. UI code calls these methods; this module does no drawing or I/O.

use super::card::{Card, DamageCard, Player, Trigger, Unit};
use super::deck::{build_deck, prepare_player};
use crate::story::Chapter;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Ride,
    Call,
    Battle,
    Cpu,
    Finished,
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

pub struct PendingAttack {
    pub cpu_attacking: bool,
    pub attacker: AttackUnit,
    pub target: AttackUnit,
    pub booster: Option<usize>,
    pub attack_power: i32,
    pub target_power: i32,
    pub guardians: Vec<Card>,
}

impl PendingAttack {
    pub fn shield(&self) -> i32 {
        self.guardians.iter().map(|card| card.shield).sum()
    }
}

pub struct MatchGame {
    pub player: Player,
    pub cpu: Player,
    pub phase: Phase,
    pub turn: u32,
    pub chapter: Chapter,
    pub message: String,
    pub log: Vec<String>,
    pub winner: Option<bool>,
    pub player_calls: u8,
    pub pending_attack: Option<PendingAttack>,
    break_ride_restand: bool,
    cpu_attack_index: usize,
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
            phase: Phase::Ride,
            turn: 1,
            chapter,
            message: "Your opening hand is ready. Choose a legal ride.".into(),
            log: Vec::new(),
            winner: None,
            player_calls: 0,
            pending_attack: None,
            break_ride_restand: false,
            cpu_attack_index: 0,
        };
        game.log("Stand up, Vanguard!");
        game
    }

    pub fn log(&mut self, line: impl Into<String>) {
        self.log.push(line.into());
        if self.log.len() > 8 {
            self.log.remove(0);
        }
    }

    pub fn ride_options(&self) -> Vec<usize> {
        let grade = self.player.grade();
        self.player
            .hand
            .iter()
            .enumerate()
            .filter(|(_, card)| {
                !card.is_trigger()
                    && card.grade >= grade
                    && card.grade <= grade + 1
                    && card.grade <= 3
            })
            .map(|(index, _)| index)
            .collect()
    }

    pub fn call_options(&self) -> Vec<usize> {
        self.player
            .hand
            .iter()
            .enumerate()
            .filter(|(_, card)| !card.is_trigger() && card.grade <= self.player.grade())
            .map(|(index, _)| index)
            .collect()
    }

    pub fn ride(&mut self, hand_index: usize) {
        if self.phase != Phase::Ride || hand_index >= self.player.hand.len() {
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
            self.message = "Ride phase complete. Call up to two rear-guards.".into();
        }
    }

    pub fn call(&mut self, hand_index: usize, rear_slot: usize) {
        if self.phase != Phase::Call
            || rear_slot >= 5
            || self.player.rear[rear_slot].is_some()
            || hand_index >= self.player.hand.len()
        {
            return;
        }
        let card = self.player.hand.remove(hand_index);
        if card.is_trigger() || card.grade > self.player.grade() {
            self.player.hand.insert(hand_index, card);
            return;
        }
        let name = card.name;
        self.player.rear[rear_slot] = Some(Unit::new(card));
        self.player_calls += 1;
        self.message = format!("Called {name} to circle {}.", rear_slot + 1);
        self.log(self.message.clone());
    }

    pub fn begin_battle(&mut self) {
        if self.phase == Phase::Call {
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
        if !attack.cpu_attacking {
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

    pub fn resolve_pending_attack(&mut self) {
        let Some(attack) = self.pending_attack.take() else {
            return;
        };
        let (attacker, defender) = if attack.cpu_attacking {
            (&mut self.cpu, &mut self.player)
        } else {
            (&mut self.player, &mut self.cpu)
        };
        let attacker_name = attack.attacker.unit(attacker).unwrap().card.name;
        let target_name = attack.target.unit(defender).unwrap().card.name;
        let outcome = resolve_attack(attacker, defender, &attack);
        defender.drop_zone.extend(attack.guardians);

        self.message = match outcome {
            AttackOutcome::Damage(damage) => {
                format!("{attacker_name} hit {target_name} for {damage} damage.")
            }
            AttackOutcome::Retired => format!("{attacker_name} retired {target_name}."),
            AttackOutcome::Miss => format!("{attacker_name}'s attack on {target_name} missed."),
        };
        self.log(self.message.clone());
        self.capture_outcome();
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
        if self.phase != Phase::Battle || self.pending_attack.is_some() {
            return;
        }
        self.phase = Phase::Cpu;
        self.cpu_attack_index = 0;
        ready_player(&mut self.cpu);
        self.cpu.draw_to_hand();
        self.capture_outcome();
        if self.phase != Phase::Cpu {
            return;
        }
        cpu_ride(&mut self.cpu);
        for _ in 0..2 {
            cpu_call(&mut self.cpu);
        }
        cpu_limit_break(&mut self.cpu, &mut self.player);
        self.advance_cpu_turn();
    }

    pub fn advance_cpu_turn(&mut self) {
        if self.phase != Phase::Cpu || self.pending_attack.is_some() {
            return;
        }
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

        unlock_rear_guards(&mut self.cpu);
        unlock_rear_guards(&mut self.player);
        self.log(format!("{} completed its turn.", self.cpu.name));
        self.start_player_turn();
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
        ready_player(&mut self.player);
        self.player.draw_to_hand();
        self.player_calls = 0;
        self.break_ride_restand = false;
        self.phase = Phase::Ride;
        self.log(format!("Turn {} — your draw phase.", self.turn));
        self.message = "New turn. Ride, call units, and make your attacks.".into();
        self.capture_outcome();
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

fn ready_player(player: &mut Player) {
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

enum AttackOutcome {
    Miss,
    Retired,
    Damage(usize),
}

fn resolve_attack(
    attacker: &mut Player,
    defender: &mut Player,
    attack: &PendingAttack,
) -> AttackOutcome {
    let is_vanguard = attack.attacker == AttackUnit::Vanguard;
    let mut critical = if is_vanguard {
        drive_check(attacker, defender.damage.len())
    } else {
        1
    };

    let attacking_unit = attack.attacker.unit(attacker).unwrap();
    let boosting_unit = attack.booster.and_then(|slot| attacker.rear[slot].as_ref());
    let attack_power = attacking_unit.power() + boosting_unit.map_or(0, Unit::power);
    let defense_power = attack.target.unit(defender).unwrap().power() + attack.shield();
    if attack_power < defense_power {
        return AttackOutcome::Miss;
    }
    if let AttackUnit::Rear(slot) = attack.target {
        let retired = defender.rear[slot].take().unwrap();
        defender.drop_zone.push(retired.card);
        return AttackOutcome::Retired;
    }

    if is_vanguard
        && attacking_unit.card.name.contains("Glory Maelstrom")
        && attacker.damage.len() >= 5
    {
        critical = 2;
    }
    AttackOutcome::Damage(deal_damage(defender, attacker, critical))
}

fn drive_check(attacker: &mut Player, opponent_damage: usize) -> i32 {
    let mut critical = 1;
    let checks = if attacker.grade() >= 3 { 2 } else { 1 };
    for _ in 0..checks {
        let Some(card) = attacker.draw() else {
            break;
        };
        if card.is_trigger() && matches_clan(attacker, &card) {
            if card.trigger == Trigger::Critical {
                critical += 1;
            }
            apply_trigger(attacker, &card, opponent_damage);
        }
        attacker.last_check = Some(format!("Drive: {}", card.name));
        attacker.hand.push(card);
    }
    critical
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

fn deal_damage(defender: &mut Player, attacker: &mut Player, count: i32) -> usize {
    let mut dealt = 0;
    for _ in 0..count {
        if defender.damage.len() >= 6 {
            break;
        }
        let Some(card) = defender.draw() else {
            break;
        };
        defender.last_check = Some(format!("Damage: {}", card.name));
        if card.is_trigger() && matches_clan(defender, &card) {
            apply_trigger(defender, &card, attacker.damage.len());
        }
        defender.damage.push(DamageCard {
            card,
            face_up: true,
        });
        dealt += 1;
    }
    dealt
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
            .any(|unit| unit.card.clan == card.clan)
}

fn apply_trigger(player: &mut Player, card: &Card, opponent_damage: usize) {
    if let Some(vanguard) = player.vanguard.as_mut() {
        vanguard.power_bonus += 5000;
    }
    match card.trigger {
        Trigger::Stand => {
            if let Some(unit) = player
                .rear
                .iter_mut()
                .flatten()
                .find(|unit| unit.rested && !unit.locked)
            {
                unit.rested = false;
            }
        }
        Trigger::Draw => player.draw_to_hand(),
        Trigger::Heal if player.damage.len() >= opponent_damage => {
            if let Some(damage) = player.damage.pop() {
                player.drop_zone.push(damage.card);
            }
        }
        _ => {}
    }
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
        game
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

        game.resolve_pending_attack();
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
            game.resolve_pending_attack();
        }
        assert_eq!(game.cpu.damage.len(), 3);
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
        assert_eq!(game.phase, Phase::Cpu);
        assert_eq!(
            game.pending_attack.as_ref().unwrap().attacker,
            AttackUnit::Rear(1)
        );
        assert!(game.player.damage.is_empty());

        game.resolve_pending_attack();
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

        game.resolve_pending_attack();
        assert_eq!(game.player.damage.len(), 1);
        assert_eq!(game.player.drop_zone.len(), 2);
        assert!(game.pending_attack.is_none());
        game.advance_cpu_turn();
        assert_eq!(
            game.pending_attack.as_ref().unwrap().attacker,
            AttackUnit::Vanguard
        );
        game.resolve_pending_attack();
        game.advance_cpu_turn();
        assert_eq!(game.phase, Phase::Ride);
        assert_eq!(game.turn, 2);
    }

    #[test]
    fn rested_front_rear_guard_can_intercept_for_another_unit() {
        let mut game = battle();
        game.cpu.rear[1] = unit("Attacking rear-guard", 2, 9000, 5000);
        game.player.rear[1] = unit("Targeted rear-guard", 2, 8000, 5000);
        game.player.rear[3] = unit("Intercepting rear-guard", 2, 9000, 5000);
        game.player.rear[3].as_mut().unwrap().rested = true;
        game.end_turn();
        assert_eq!(
            game.pending_attack.as_ref().unwrap().target,
            AttackUnit::Rear(1)
        );
        assert!(!game.can_intercept(1));
        assert!(game.intercept(3));
        assert!(game.player.rear[3].is_none());
        assert_eq!(game.pending_attack.as_ref().unwrap().shield(), 5000);
        assert!(game.player.drop_zone.is_empty());
        game.resolve_pending_attack();
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

        game.resolve_pending_attack();
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
            game.resolve_pending_attack();
            assert!(!game.can_attack(rear_guard));

            game.player
                .deck
                .push(Card::trigger("Drive trigger", "Royal Paladin", kind));
            assert!(game.declare_attack(AttackUnit::Vanguard, AttackUnit::Vanguard, false));
            game.resolve_pending_attack();

            assert_eq!(game.can_attack(rear_guard), kind == Trigger::Stand);
            assert_eq!(
                game.cpu.damage.len(),
                if kind == Trigger::Critical { 2 } else { 1 }
            );
        }
    }
}
