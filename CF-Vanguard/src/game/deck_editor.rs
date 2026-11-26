//! Catalog filtering and classic-era deck editing, independent of rendering.

use super::card::Card;
use super::deck::{card_from_record, shuffle};
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;

pub const NATIONS: [&str; 6] = [
    "United Sanctuary",
    "Dragon Empire",
    "Star Gate",
    "Dark Zone",
    "Magallanica",
    "Zoo",
];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CardSort {
    #[default]
    Nation,
    Name,
    Grade,
    Power,
    Number,
}

impl CardSort {
    pub const ALL: [Self; 5] = [
        Self::Nation,
        Self::Name,
        Self::Grade,
        Self::Power,
        Self::Number,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Nation => "Nation / Clan / Grade",
            Self::Name => "Name A - Z",
            Self::Grade => "Grade: low to high",
            Self::Power => "Power: high to low",
            Self::Number => "Card number",
        }
    }
}

#[derive(Default)]
pub struct CatalogFilter {
    pub query: String,
    pub nation: Option<&'static str>,
    pub clan: Option<String>,
    /// A zero mask shows every grade; selected grades are combined with OR.
    pub grades: u8,
    pub sort: CardSort,
}

impl CatalogFilter {
    pub fn results(&self, records: &[Value]) -> Vec<usize> {
        let query = self.query.to_lowercase();
        let mut result: Vec<_> = records
            .iter()
            .enumerate()
            .filter_map(|(i, card)| {
                let grade = number(card, "grade");
                let matches = self.nation.is_none_or(|n| nation(card) == n)
                    && self.clan.as_ref().is_none_or(|c| field(card, "clan") == c)
                    && (self.grades == 0
                        || (0..8).contains(&grade) && self.grades & (1 << grade) != 0)
                    && (query.is_empty()
                        || [
                            "card_name",
                            "card_number",
                            "clan",
                            "country",
                            "product",
                            "text",
                        ]
                        .iter()
                        .any(|key| field(card, key).to_lowercase().contains(&query)));
                matches.then_some(i)
            })
            .collect();
        result.sort_by(|a, b| {
            let a = &records[*a];
            let b = &records[*b];
            let order = match self.sort {
                CardSort::Nation => hierarchy(a).cmp(&hierarchy(b)),
                CardSort::Name => field(a, "card_name")
                    .to_lowercase()
                    .cmp(&field(b, "card_name").to_lowercase()),
                CardSort::Grade => number(a, "grade").cmp(&number(b, "grade")),
                CardSort::Power => number(b, "power").cmp(&number(a, "power")),
                CardSort::Number => field(a, "card_number").cmp(field(b, "card_number")),
            };
            order
                .then_with(|| field(a, "card_name").cmp(field(b, "card_name")))
                .then_with(|| field(a, "card_number").cmp(field(b, "card_number")))
        });
        result
    }
}

pub fn field<'a>(record: &'a Value, key: &str) -> &'a str {
    record[key].as_str().unwrap_or("")
}

/// The source catalog incorrectly assigns fourteen Spike Brothers printings to Zoo.
pub fn nation(card: &Value) -> &str {
    if field(card, "clan") == "Spike Brothers" {
        "Dark Zone"
    } else {
        field(card, "country")
    }
}

pub fn number(record: &Value, key: &str) -> i32 {
    field(record, key).parse().unwrap_or(0)
}

fn hierarchy(card: &Value) -> (usize, &str, i32) {
    (
        NATIONS
            .iter()
            .position(|n| *n == nation(card))
            .unwrap_or(NATIONS.len()),
        field(card, "clan"),
        number(card, "grade"),
    )
}

pub fn is_trigger(card: &Value) -> bool {
    ["Critical", "Draw", "Stand", "Heal"]
        .iter()
        .any(|kind| field(card, "card_trigger").contains(kind))
}

#[derive(Default)]
pub struct EditedDeck {
    pub cards: Vec<usize>,
}

impl EditedDeck {
    pub fn copies(&self, index: usize, records: &[Value]) -> usize {
        self.cards
            .iter()
            .filter(|i| field(&records[**i], "card_name") == field(&records[index], "card_name"))
            .count()
    }

    pub fn add(&mut self, index: usize, records: &[Value]) -> Result<(), &'static str> {
        if index >= records.len() {
            return Err("Card is unavailable.");
        }
        if self.cards.len() >= 50 {
            return Err("Deck full. Remove a card first.");
        }
        if self.copies(index, records) >= 4 {
            return Err("Four copies maximum, including other printings.");
        }
        if self
            .cards
            .first()
            .is_some_and(|first| field(&records[*first], "clan") != field(&records[index], "clan"))
        {
            return Err("Choose cards from the same clan, or start a new deck.");
        }
        self.cards.push(index);
        Ok(())
    }

    pub fn remove(&mut self, index: usize) {
        if let Some(position) = self.cards.iter().rposition(|i| *i == index) {
            self.cards.remove(position);
        }
    }

    pub fn validation(&self, records: &[Value]) -> Result<(), &'static str> {
        if self.cards.iter().any(|i| *i >= records.len()) {
            return Err("Deck contains unavailable cards.");
        }
        if self.cards.len() != 50 {
            return Err("Build a 50-card deck to use it in Quick Fight.");
        }
        let clan = field(&records[self.cards[0]], "clan");
        let mut counts = HashMap::new();
        let mut triggers = 0;
        let mut heals = 0;
        let mut starter = false;
        for i in &self.cards {
            let card = &records[*i];
            if field(card, "clan") != clan {
                return Err("All cards must belong to the same clan.");
            }
            let count = counts.entry(field(card, "card_name")).or_insert(0);
            *count += 1;
            if *count > 4 {
                return Err("Four copies maximum, including other printings.");
            }
            triggers += usize::from(is_trigger(card));
            heals += usize::from(field(card, "card_trigger").contains("Heal"));
            starter |= number(card, "grade") == 0 && !is_trigger(card);
        }
        if triggers != 16 {
            return Err("A classic deck needs exactly 16 triggers.");
        }
        if heals > 4 {
            return Err("Use no more than four Heal triggers.");
        }
        if !starter {
            return Err("Add a non-trigger Grade 0 starting Vanguard.");
        }
        Ok(())
    }

    pub fn playable(&self, records: &[Value]) -> Result<Vec<Card>, &'static str> {
        self.validation(records)?;
        let mut cards = self
            .cards
            .iter()
            .map(|i| card_from_record(&records[*i]))
            .collect::<Vec<_>>();
        shuffle(&mut cards);
        Ok(cards)
    }

    pub fn sample(clan: &str, records: &[Value]) -> Self {
        let pool = records
            .iter()
            .enumerate()
            .filter(|(_, r)| field(r, "clan") == clan)
            .map(|(i, _)| i)
            .collect::<Vec<_>>();
        let mut deck = Self::default();
        if let Some(starter) = pool
            .iter()
            .find(|i| number(&records[**i], "grade") == 0 && !is_trigger(&records[**i]))
        {
            deck.cards.push(*starter);
        }
        // Prefer four of each trigger type; clans without a type use another named trigger.
        for kind in ["Critical", "Draw", "Stand", "Heal"] {
            if let Some(index) = pool
                .iter()
                .find(|i| field(&records[**i], "card_trigger").contains(kind))
            {
                for _ in 0..4 {
                    let _ = deck.add(*index, records);
                }
            }
        }
        for index in pool.iter().filter(|i| {
            is_trigger(&records[**i]) && !field(&records[**i], "card_trigger").contains("Heal")
        }) {
            while deck.cards.len() < 17 && deck.copies(*index, records) < 4 {
                let _ = deck.add(*index, records);
            }
        }
        for (grade, target) in [(1, 14), (2, 11), (3, 8)] {
            let mut remaining = target;
            for index in pool
                .iter()
                .filter(|i| number(&records[**i], "grade") == grade)
            {
                while remaining > 0 && deck.copies(*index, records) < 4 {
                    if deck.add(*index, records).is_err() {
                        break;
                    }
                    remaining -= 1;
                }
            }
        }
        deck
    }

    pub fn save(&self, records: &[Value]) -> Result<(), String> {
        self.validation(records).map_err(str::to_owned)?;
        let path = saved_deck_path().ok_or("Cannot locate the user data folder.")?;
        std::fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
        let numbers: Vec<_> = self
            .cards
            .iter()
            .map(|i| field(&records[*i], "card_number"))
            .collect();
        let json = serde_json::to_string_pretty(&numbers).map_err(|e| e.to_string())?;
        let temporary = path.with_extension("tmp");
        std::fs::write(&temporary, json).map_err(|e| e.to_string())?;
        std::fs::rename(temporary, path).map_err(|e| e.to_string())
    }

    pub fn load(records: &[Value]) -> Result<Option<Self>, String> {
        let Some(path) = saved_deck_path() else {
            return Ok(None);
        };
        let json = match std::fs::read_to_string(path) {
            Ok(json) => json,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(e.to_string()),
        };
        Self::from_saved_json(&json, records).map(Some)
    }

    fn from_saved_json(json: &str, records: &[Value]) -> Result<Self, String> {
        let numbers: Vec<String> = serde_json::from_str(&json).map_err(|e| e.to_string())?;
        let cards: Option<Vec<_>> = numbers
            .iter()
            .map(|n| records.iter().position(|r| field(r, "card_number") == n))
            .collect();
        let deck = Self {
            cards: cards.ok_or("Saved deck contains unavailable cards.")?,
        };
        deck.validation(records).map_err(str::to_owned)?;
        Ok(deck)
    }
}

fn saved_deck_path() -> Option<PathBuf> {
    if cfg!(target_os = "macos") {
        Some(
            PathBuf::from(std::env::var_os("HOME")?)
                .join("Library/Application Support/CF-Vanguard/quick-fight-deck.json"),
        )
    } else if cfg!(target_os = "windows") {
        Some(PathBuf::from(std::env::var_os("APPDATA")?).join("CF-Vanguard/quick-fight-deck.json"))
    } else {
        let root = std::env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|p| PathBuf::from(p).join(".local/share")))?;
        Some(root.join("CF-Vanguard/quick-fight-deck.json"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn catalog() -> Vec<Value> {
        serde_json::from_str::<Value>(include_str!("../../assets/official_era_cards.json")).unwrap()
            ["cards"]
            .as_array()
            .unwrap()
            .clone()
    }

    #[test]
    fn filters_intersect_and_grades_combine() {
        let records = catalog();
        let mut filter = CatalogFilter::default();
        assert_eq!(filter.results(&records).len(), records.len());
        filter.nation = Some("Dragon Empire");
        filter.clan = Some("Kagero".into());
        filter.grades = (1 << 1) | (1 << 3);
        let result = filter.results(&records);
        assert!(!result.is_empty());
        assert!(result
            .iter()
            .all(|i| field(&records[*i], "clan") == "Kagero"
                && [1, 3].contains(&number(&records[*i], "grade"))));
        let expected = result[0];
        filter.query = field(&records[expected], "card_number").into();
        assert_eq!(filter.results(&records), vec![expected]);
        filter.nation = Some("Zoo");
        assert!(filter.results(&records).is_empty());
    }

    #[test]
    fn sorts_are_numeric_and_default_preserves_hierarchy() {
        let records = catalog();
        for sort in CardSort::ALL {
            let filter = CatalogFilter {
                sort,
                ..Default::default()
            };
            let results = filter.results(&records);
            assert_eq!(results.len(), records.len());
            for pair in results.windows(2) {
                let a = &records[pair[0]];
                let b = &records[pair[1]];
                match sort {
                    CardSort::Nation => assert!(hierarchy(a) <= hierarchy(b)),
                    CardSort::Grade => assert!(number(a, "grade") <= number(b, "grade")),
                    CardSort::Power => assert!(number(a, "power") >= number(b, "power")),
                    CardSort::Name => assert!(
                        field(a, "card_name").to_lowercase()
                            <= field(b, "card_name").to_lowercase()
                    ),
                    CardSort::Number => assert!(field(a, "card_number") <= field(b, "card_number")),
                }
            }
        }
    }

    #[test]
    fn reprints_share_copy_limit_and_mixed_clans_are_rejected() {
        let records = catalog();
        let first = records
            .iter()
            .position(|r| field(r, "card_name") == "Dragonic Overlord")
            .unwrap();
        let reprint = records
            .iter()
            .enumerate()
            .find(|(i, r)| *i != first && field(r, "card_name") == "Dragonic Overlord")
            .unwrap()
            .0;
        let mut deck = EditedDeck::default();
        for _ in 0..4 {
            deck.add(first, &records).unwrap();
        }
        assert!(deck.add(reprint, &records).is_err());
        let other = records
            .iter()
            .position(|r| field(r, "clan") == "Royal Paladin")
            .unwrap();
        assert!(deck.add(other, &records).is_err());
        deck.remove(first);
        deck.add(reprint, &records).unwrap();
        assert_eq!(deck.cards.len(), 4);
    }

    #[test]
    fn nation_filter_corrects_spike_brothers_source_errors() {
        let records = catalog();
        let mut filter = CatalogFilter {
            nation: Some("Dark Zone"),
            clan: Some("Spike Brothers".into()),
            ..Default::default()
        };
        assert_eq!(filter.results(&records).len(), 44);
        filter.nation = Some("Zoo");
        assert!(filter.results(&records).is_empty());
    }

    #[test]
    fn saved_decks_round_trip_and_reject_missing_or_invalid_cards() {
        let records = catalog();
        let deck = EditedDeck::sample("Kagero", &records);
        let numbers = deck
            .cards
            .iter()
            .map(|i| field(&records[*i], "card_number"))
            .collect::<Vec<_>>();
        let json = serde_json::to_string(&numbers).unwrap();
        let restored = EditedDeck::from_saved_json(&json, &records).unwrap();
        assert_eq!(restored.cards, deck.cards);
        assert!(EditedDeck::from_saved_json("[]", &records).is_err());
        assert!(EditedDeck::from_saved_json("[\"MISSING\"]", &records).is_err());
        assert!(EditedDeck::from_saved_json("broken", &records).is_err());
        let game = super::super::engine::MatchGame::with_player_deck(
            crate::story::CHAPTERS[0],
            restored.playable(&records).unwrap(),
        );
        assert_eq!(game.player.hand.len(), 5);
        assert_eq!(game.player.deck.len(), 44);
        assert_eq!(game.player.vanguard.as_ref().unwrap().card.clan, "Kagero");
        assert!(game
            .player
            .hand
            .iter()
            .chain(game.player.deck.iter())
            .all(|c| c.clan == "Kagero"));
    }

    #[test]
    fn samples_validate_and_playable_decks_keep_card_stats() {
        let records = catalog();
        for clan in [
            "Royal Paladin",
            "Kagero",
            "Aqua Force",
            "Angel Feather",
            "Bermuda Triangle",
            "Dark Irregulars",
            "Dimension Police",
            "Genesis",
            "Gold Paladin",
            "Granblue",
            "Great Nature",
            "Link Joker",
            "Megacolony",
            "Murakumo",
            "Narukami",
            "Neo Nectar",
            "Nova Grappler",
            "Nubatama",
            "Oracle Think Tank",
            "Pale Moon",
            "Shadow Paladin",
            "Spike Brothers",
            "Tachikaze",
        ] {
            let deck = EditedDeck::sample(clan, &records);
            assert_eq!(deck.validation(&records), Ok(()), "{clan}");
            let cards = deck.playable(&records).unwrap();
            assert_eq!(cards.len(), 50);
            assert_eq!(cards.iter().filter(|c| c.is_trigger()).count(), 16);
        }
        let mut deck = EditedDeck::sample("Kagero", &records);
        deck.cards.pop();
        assert!(deck.validation(&records).is_err());
    }
}
