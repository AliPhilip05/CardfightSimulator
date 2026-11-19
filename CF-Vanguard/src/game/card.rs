//! Card, trigger, unit, and player data used by the match engine.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Trigger {
    None,
    Critical,
    Draw,
    Stand,
    Heal,
}

#[derive(Clone, Debug)]
pub struct Card {
    pub name: &'static str,
    pub clan: &'static str,
    pub grade: u8,
    pub power: i32,
    pub shield: i32,
    pub trigger: Trigger,
    pub break_ride: bool,
    pub limit_break: bool,
}

impl Card {
    pub fn unit(
        name: &'static str,
        clan: &'static str,
        grade: u8,
        power: i32,
        shield: i32,
    ) -> Self {
        Self {
            name,
            clan,
            grade,
            power,
            shield,
            trigger: Trigger::None,
            break_ride: false,
            limit_break: false,
        }
    }

    pub fn trigger(name: &'static str, clan: &'static str, kind: Trigger) -> Self {
        Self {
            name,
            clan,
            grade: 0,
            power: 5000,
            shield: 10000,
            trigger: kind,
            break_ride: false,
            limit_break: false,
        }
    }

    pub fn is_trigger(&self) -> bool {
        self.trigger != Trigger::None
    }
}

#[derive(Clone, Debug)]
pub struct Unit {
    pub card: Card,
    pub rested: bool,
    pub power_bonus: i32,
    pub locked: bool,
}

impl Unit {
    pub fn new(card: Card) -> Self {
        Self {
            card,
            rested: false,
            power_bonus: 0,
            locked: false,
        }
    }

    pub fn power(&self) -> i32 {
        self.card.power + self.power_bonus
    }
}

#[derive(Clone, Debug)]
pub struct DamageCard {
    pub card: Card,
    pub face_up: bool,
}

#[derive(Clone, Debug)]
pub struct Player {
    pub name: String,
    pub deck: Vec<Card>,
    pub hand: Vec<Card>,
    pub vanguard: Option<Unit>,
    /// Slots: left-back, left-front, center-back, right-front, right-back.
    pub rear: Vec<Option<Unit>>,
    pub damage: Vec<DamageCard>,
    pub soul: Vec<Card>,
    pub drop_zone: Vec<Card>,
    pub last_check: Option<String>,
    pub deck_out: bool,
}

impl Player {
    pub fn new(name: impl Into<String>, deck: Vec<Card>) -> Self {
        Self {
            name: name.into(),
            deck,
            hand: Vec::new(),
            vanguard: None,
            rear: vec![None; 5],
            damage: Vec::new(),
            soul: Vec::new(),
            drop_zone: Vec::new(),
            last_check: None,
            deck_out: false,
        }
    }

    pub fn draw(&mut self) -> Option<Card> {
        let card = self.deck.pop();
        self.deck_out = card.is_none();
        card
    }

    pub fn draw_to_hand(&mut self) {
        if let Some(card) = self.draw() {
            self.hand.push(card);
        }
    }

    pub fn grade(&self) -> u8 {
        self.vanguard.as_ref().map_or(0, |unit| unit.card.grade)
    }

    pub fn face_up_damage(&self) -> usize {
        self.damage.iter().filter(|card| card.face_up).count()
    }

    pub fn vanguard_power(&self) -> i32 {
        self.vanguard.as_ref().map_or(0, Unit::power)
    }
}
