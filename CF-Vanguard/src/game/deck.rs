//! Builds 50-card sample decks from the bundled official-era card catalog.

use super::card::{Card, Player, Trigger};
use serde_json::Value;
use std::time::{SystemTime, UNIX_EPOCH};

const CARD_DATA: &str = include_str!("../../assets/official_era_cards.json");

/// Builds a playable clan list from real card names and stats.
///
/// The catalog includes English BT01-BT15, EB01-EB10, and TD01-TD13 entries.
/// This produces a consistent sample list, not an exact copy of a 3DS deck recipe.
pub fn build_deck(clan: &'static str, boss: &'static str, break_ride: bool) -> Vec<Card> {
    let root: Value = serde_json::from_str(CARD_DATA).expect("bundled card catalog is valid JSON");
    let cards = root["cards"].as_array().expect("catalog has cards");
    let clan_cards: Vec<&Value> = cards
        .iter()
        .filter(|card| card["clan"].as_str() == Some(clan))
        .collect();

    let starter = find_starter(&clan_cards).unwrap_or_else(|| {
        // Every story clan has a grade-zero unit in the bundled era card pool.
        panic!("no grade-zero starter in official catalog for {clan}")
    });
    let mut deck = vec![to_unit(starter, clan)];

    for (kind, trigger) in [
        ("Critical", Trigger::Critical),
        ("Draw", Trigger::Draw),
        ("Stand", Trigger::Stand),
        ("Heal", Trigger::Heal),
    ] {
        let card = clan_cards
            .iter()
            .copied()
            .find(|card| card["card_trigger"].as_str().unwrap_or("").contains(kind));
        if let Some(card) = card {
            for _ in 0..4 {
                deck.push(to_trigger(card, clan, trigger));
            }
        }
    }

    add_grade_group(&mut deck, &clan_cards, clan, 1, 17, None);
    add_grade_group(&mut deck, &clan_cards, clan, 2, 11, None);

    if break_ride {
        if let Some(unit) = find_break_ride(&clan_cards, clan) {
            for _ in 0..4 {
                let mut card = to_unit(unit, clan);
                card.break_ride = true;
                card.limit_break = true;
                deck.push(card);
            }
            let boss_card =
                find_named(&clan_cards, boss).or_else(|| find_grade(&clan_cards, 3, None));
            if let Some(card) = boss_card {
                let mut card = to_unit(card, clan);
                card.limit_break = true;
                deck.push(card);
            }
        } else {
            add_boss_group(&mut deck, &clan_cards, clan, boss);
        }
    } else {
        add_boss_group(&mut deck, &clan_cards, clan, boss);
    }

    // Keep this prototype deck at 50 when a clan lacks one of the four trigger types.
    if deck.len() < 50 {
        if let Some(card) = clan_cards.iter().copied().find(|card| is_trigger(card)) {
            while deck.len() < 50 {
                let trigger = trigger_kind(card);
                deck.push(to_trigger(card, clan, trigger));
            }
        }
    }
    deck.truncate(50);
    shuffle(&mut deck);
    deck
}

fn add_boss_group(deck: &mut Vec<Card>, cards: &[&Value], clan: &'static str, boss: &str) {
    let boss_card = find_named(cards, boss).or_else(|| find_grade(cards, 3, None));
    if let Some(card) = boss_card {
        for _ in 0..4 {
            let mut unit = to_unit(card, clan);
            unit.limit_break = true;
            deck.push(unit);
        }
    }
    if let Some(second) = find_grade(cards, 3, boss_card.map(card_number)) {
        let mut unit = to_unit(second, clan);
        unit.limit_break = true;
        deck.push(unit);
    }
}

fn add_grade_group(
    deck: &mut Vec<Card>,
    cards: &[&Value],
    clan: &'static str,
    grade: u8,
    target: usize,
    _exclude: Option<&str>,
) {
    let picks = unique_units(cards, grade);
    let mut remaining = target;
    for card in picks {
        let copies = remaining.min(4);
        for _ in 0..copies {
            deck.push(to_unit(card, clan));
        }
        remaining -= copies;
        if remaining == 0 {
            break;
        }
    }
}

fn find_starter<'a>(cards: &[&'a Value]) -> Option<&'a Value> {
    cards
        .iter()
        .copied()
        .find(|card| grade(card) == Some(0) && !is_trigger(card))
}

fn find_grade<'a>(
    cards: &[&'a Value],
    desired: u8,
    exclude_number: Option<&str>,
) -> Option<&'a Value> {
    cards.iter().copied().find(|card| {
        grade(card) == Some(desired)
            && !is_trigger(card)
            && exclude_number != Some(card_number(card))
    })
}

fn find_named<'a>(cards: &[&'a Value], name: &str) -> Option<&'a Value> {
    let target = normalize_name(name);
    cards.iter().copied().find(|card| {
        card["card_name"]
            .as_str()
            .is_some_and(|card_name| normalize_name(card_name) == target)
    })
}

fn normalize_name(name: &str) -> String {
    name.chars()
        .filter(|character| character.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

fn find_break_ride<'a>(cards: &[&'a Value], clan: &str) -> Option<&'a Value> {
    let name = match clan {
        "Kagero" => "Dauntless Drive Dragon",
        "Narukami" => "Eradicator, Vowing Sword Dragon",
        "Link Joker" => "Star-vader, Infinite Zero Dragon",
        _ => return None,
    };
    find_named(cards, name)
}

fn unique_units<'a>(cards: &[&'a Value], desired_grade: u8) -> Vec<&'a Value> {
    cards
        .iter()
        .copied()
        .filter(|card| grade(card) == Some(desired_grade) && !is_trigger(card))
        .collect()
}

fn grade(card: &Value) -> Option<u8> {
    card["grade"].as_str()?.parse().ok()
}

fn card_number(card: &Value) -> &str {
    card["card_number"].as_str().unwrap_or("")
}

fn is_trigger(card: &Value) -> bool {
    let trigger = card["card_trigger"].as_str().unwrap_or("");
    trigger.contains("Critical")
        || trigger.contains("Draw")
        || trigger.contains("Stand")
        || trigger.contains("Heal")
}

fn trigger_kind(card: &Value) -> Trigger {
    let value = card["card_trigger"].as_str().unwrap_or("");
    if value.contains("Critical") {
        Trigger::Critical
    } else if value.contains("Draw") {
        Trigger::Draw
    } else if value.contains("Stand") {
        Trigger::Stand
    } else {
        Trigger::Heal
    }
}

fn to_unit(card: &Value, clan: &'static str) -> Card {
    let name = leaked(card["card_name"].as_str().unwrap_or("Unknown Card"));
    let actual_clan = leaked(card["clan"].as_str().unwrap_or(clan));
    Card::unit(
        name,
        actual_clan,
        grade(card).unwrap_or(0),
        card["power"]
            .as_str()
            .and_then(|value| value.parse().ok())
            .unwrap_or(5000),
        card["shield"]
            .as_str()
            .and_then(|value| value.parse().ok())
            .unwrap_or(0),
    )
}

fn to_trigger(card: &Value, clan: &'static str, kind: Trigger) -> Card {
    let mut result = Card::trigger(
        leaked(card["card_name"].as_str().unwrap_or("Trigger Unit")),
        leaked(card["clan"].as_str().unwrap_or(clan)),
        kind,
    );
    result.power = card["power"]
        .as_str()
        .and_then(|value| value.parse().ok())
        .unwrap_or(5000);
    result.shield = card["shield"]
        .as_str()
        .and_then(|value| value.parse().ok())
        .unwrap_or(10000);
    result
}

fn leaked(value: &str) -> &'static str {
    Box::leak(value.to_owned().into_boxed_str())
}

pub fn prepare_player(name: impl Into<String>, deck: Vec<Card>) -> Player {
    let mut player = Player::new(name, deck);
    let starter_index = player
        .deck
        .iter()
        .position(|card| card.grade == 0 && !card.is_trigger())
        .unwrap_or(0);
    let starter = player.deck.remove(starter_index);
    player.vanguard = Some(super::card::Unit::new(starter));
    for _ in 0..5 {
        player.draw_to_hand();
    }
    player
}

pub(super) fn shuffle<T>(items: &mut [T]) {
    let mut seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64;
    for index in (1..items.len()).rev() {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        items.swap(index, (seed as usize) % (index + 1));
    }
}
