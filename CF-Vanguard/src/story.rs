//! Story chapters and the three Link Joker-era school routes.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SchoolRoute {
    Hitsue,
    Fukuhara,
    Miyaji,
}

impl SchoolRoute {
    pub const ALL: [Self; 3] = [Self::Hitsue, Self::Fukuhara, Self::Miyaji];

    pub fn name(self) -> &'static str {
        match self {
            Self::Hitsue => "Hitsue High",
            Self::Fukuhara => "Fukuhara High",
            Self::Miyaji => "Miyaji Academy",
        }
    }

    pub fn rival(self) -> (&'static str, &'static str, &'static str) {
        match self {
            Self::Hitsue => (
                "Reversed Toshiki Kai",
                "Kagero",
                "Dragonic Overlord The Re-birth",
            ),
            Self::Fukuhara => (
                "Reversed Asaka Narumi",
                "Pale Moon",
                "Silver Thorn Dragon Queen, Luquier Яeverse",
            ),
            Self::Miyaji => (
                "Reversed Naoki Ishida",
                "Narukami",
                "Eradicator, Vowing Saber Dragon Яeverse",
            ),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Chapter {
    pub title: &'static str,
    pub synopsis: &'static str,
    pub rival: &'static str,
    pub rival_clan: &'static str,
    pub rival_boss: &'static str,
    pub player_clan: &'static str,
    pub player_boss: &'static str,
    pub break_ride: bool,
    pub reward: &'static str,
}

pub const CHAPTERS: [Chapter; 6] = [
    Chapter {
        title: "Card Capital: First Stand",
        synopsis: "Aichi and the Card Capital regulars introduce a newcomer to Vanguard. Learn the ride pattern and take your first cardfight.",
        rival: "Katsumi Morikawa",
        rival_clan: "Kagero",
        rival_boss: "Dragonic Overlord",
        player_clan: "Royal Paladin",
        player_boss: "Blaster Blade",
        break_ride: false,
        reward: "Starter cards and tournament entry",
    },
    Chapter {
        title: "The Shop Tournament",
        synopsis: "The shop bracket raises the stakes. Manage damage and save Counter-Blast for a Limit Break turn.",
        rival: "Toshiki Kai",
        rival_clan: "Kagero",
        rival_boss: "Dragonic Overlord The End",
        player_clan: "Kagero",
        player_boss: "Dragonic Overlord",
        break_ride: false,
        reward: "Limit Break cards from the BT09 era",
    },
    Chapter {
        title: "Asia Circuit: A New Team",
        synopsis: "Aichi's team meets powerful international fighters and faces high-level Limit Break decks.",
        rival: "Leon Soryu",
        rival_clan: "Aqua Force",
        rival_boss: "Blue Storm Dragon, Maelstrom",
        player_clan: "Gold Paladin",
        player_boss: "Blazing Lion, Platina Ezel",
        break_ride: false,
        reward: "Crossrides and Ultimate Break",
    },
    Chapter {
        title: "VF Koshien: Cardfight Club",
        synopsis: "Aichi builds a high-school Cardfight Club. Hitsue, Fukuhara, and Miyaji aim for VF Koshien as Break Ride cards arrive.",
        rival: "Ren Suzugamori",
        rival_clan: "Shadow Paladin",
        rival_boss: "Illusionary Revenger, Mordred Phantom",
        player_clan: "Kagero",
        player_boss: "Dragonic Overlord",
        break_ride: true,
        reward: "Break Ride cards from TD08, TD09, and BT11",
    },
    Chapter {
        title: "The Black Ring: Reversed",
        synopsis: "A strange black ring spreads across the schools. Link Joker's Lock begins to reshape the fight.",
        rival: "Reversed Naoki Ishida",
        rival_clan: "Narukami",
        rival_boss: "Eradicator, Vowing Saber Dragon Яeverse",
        player_clan: "Narukami",
        player_boss: "Eradicator, Vowing Saber Dragon Яeverse",
        break_ride: true,
        reward: "Lock, Unlock, and Reverse cards from BT12 onward",
    },
    Chapter {
        title: "Final Turn: Star-vader",
        synopsis: "The Reverse crisis escalates toward Void. Face the Star-vaders and the BT15-era finale.",
        rival: "Reversed Takuto Tatsunagi",
        rival_clan: "Link Joker",
        rival_boss: "Star-vader, \"Omega\" Glendios",
        player_clan: "Link Joker",
        player_boss: "Star-vader, Nebula Lord Dragon",
        break_ride: true,
        reward: "Omega Glendios and the BT15 finale",
    },
];
