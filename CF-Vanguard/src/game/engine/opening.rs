//! Pre-fight choices and a single bottom-return, draw, then shuffle mulligan.
use super::*;
use crate::game::deck::shuffle;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RpsChoice {
    Rock,
    Paper,
    Scissors,
}
impl RpsChoice {
    pub const ALL: [Self; 3] = [Self::Rock, Self::Paper, Self::Scissors];
    pub fn label(self) -> &'static str {
        match self {
            Self::Rock => "ROCK",
            Self::Paper => "PAPER",
            Self::Scissors => "SCISSORS",
        }
    }
    pub fn beats(self, other: Self) -> bool {
        matches!(
            (self, other),
            (Self::Rock, Self::Scissors)
                | (Self::Paper, Self::Rock)
                | (Self::Scissors, Self::Paper)
        )
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SetupStage {
    RockPaperScissors,
    Reveal,
    ChooseOrder,
    Mulligan,
    HandsReady,
    StandUp,
    Complete,
}

impl MatchGame {
    pub fn play_rps(&mut self, player: RpsChoice, cpu: RpsChoice) -> bool {
        if self.phase != Phase::Setup || self.setup_stage != SetupStage::RockPaperScissors {
            return false;
        }
        self.rps_result = Some((player, cpu));
        self.player_won_rps = (player != cpu).then_some(player.beats(cpu));
        self.setup_stage = SetupStage::Reveal;
        true
    }
    pub fn finish_rps_reveal(&mut self) {
        if self.phase != Phase::Setup || self.setup_stage != SetupStage::Reveal {
            return;
        }
        self.setup_stage = match self.player_won_rps {
            None => SetupStage::RockPaperScissors,
            Some(true) => SetupStage::ChooseOrder,
            Some(false) => {
                self.player_first = false;
                SetupStage::Mulligan
            }
        };
    }
    pub fn choose_turn_order(&mut self, first: bool) -> bool {
        if self.phase != Phase::Setup || self.setup_stage != SetupStage::ChooseOrder {
            return false;
        }
        self.player_first = first;
        self.setup_stage = SetupStage::Mulligan;
        true
    }
    pub fn mulligan(&mut self, indices: &[usize]) -> bool {
        if self.phase != Phase::Setup
            || self.setup_stage != SetupStage::Mulligan
            || !valid_mulligan(&self.player, indices)
        {
            return false;
        }
        let cpu_indices = cpu_mulligan(&self.cpu);
        if self.player_first {
            replace_opening_hand(&mut self.player, indices);
            replace_opening_hand(&mut self.cpu, &cpu_indices);
        } else {
            replace_opening_hand(&mut self.cpu, &cpu_indices);
            replace_opening_hand(&mut self.player, indices);
        }
        self.log(format!(
            "Opening hands ready. You replaced {}; {} replaced {}.",
            indices.len(),
            self.cpu.name,
            cpu_indices.len()
        ));
        self.setup_stage = SetupStage::HandsReady;
        true
    }
    pub fn confirm_opening_hands(&mut self) -> bool {
        if self.phase != Phase::Setup || self.setup_stage != SetupStage::HandsReady {
            return false;
        }
        self.setup_stage = SetupStage::StandUp;
        self.log("Stand up, Vanguard!");
        true
    }
    pub fn begin_fight(&mut self) {
        if self.phase != Phase::Setup || self.setup_stage != SetupStage::StandUp {
            return;
        }
        self.setup_stage = SetupStage::Complete;
        self.phase = if self.player_first {
            Phase::Stand
        } else {
            Phase::Cpu
        };
        self.cpu_stage = TurnStage::Stand;
    }
}
fn valid_mulligan(player: &Player, indices: &[usize]) -> bool {
    indices.len() <= 5
        && indices.iter().all(|&i| i < player.hand.len())
        && indices
            .iter()
            .enumerate()
            .all(|(i, index)| !indices[..i].contains(index))
}
fn replace_opening_hand(player: &mut Player, indices: &[usize]) {
    let mut sorted = indices.to_vec();
    sorted.sort_unstable();
    let mut returned = Vec::new();
    for &i in sorted.iter().rev() {
        returned.push(player.hand.remove(i));
    }
    returned.reverse();
    // Vec::pop is the top: index zero is the bottom. Draw before shuffling so a returned copy cannot immediately come back.
    let count = returned.len();
    player.deck.splice(0..0, returned);
    for _ in 0..count {
        player.draw_to_hand();
    }
    if count > 0 {
        shuffle(&mut player.deck);
    }
}
fn cpu_mulligan(player: &Player) -> Vec<usize> {
    let mut kept = [false; 4];
    player
        .hand
        .iter()
        .enumerate()
        .filter_map(|(i, c)| {
            let grade = c.grade as usize;
            if (1..=3).contains(&grade) && !kept[grade] {
                kept[grade] = true;
                None
            } else {
                Some(i)
            }
        })
        .collect()
}
