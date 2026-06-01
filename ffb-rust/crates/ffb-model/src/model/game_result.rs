use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::model::player::PlayerId;

/// Per-player game result (accumulated during the game).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlayerResult {
    pub touchdowns: i32,
    pub completions: i32,
    pub casualties: i32,
    pub interceptions: i32,
    pub deflections: i32,
    pub mvp: bool,
    pub spp_gained: i32,
}

/// Per-team final game result.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TeamResult {
    pub score: i32,
    pub winnings: i32,
    pub fan_factor_modifier: i32,
    pub player_results: HashMap<PlayerId, PlayerResult>,
}

/// Full game result.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GameResult {
    pub home: TeamResult,
    pub away: TeamResult,
}

impl GameResult {
    pub fn winner(&self) -> Option<bool> {
        match self.home.score.cmp(&self.away.score) {
            std::cmp::Ordering::Greater => Some(true),  // home wins
            std::cmp::Ordering::Less => Some(false),    // away wins
            std::cmp::Ordering::Equal => None,          // draw
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn winner_logic() {
        let mut gr = GameResult::default();
        gr.home.score = 2;
        gr.away.score = 1;
        assert_eq!(gr.winner(), Some(true));

        gr.home.score = 1;
        assert_eq!(gr.winner(), None);

        gr.away.score = 2;
        assert_eq!(gr.winner(), Some(false));
    }

    #[test]
    fn serde_round_trip() {
        let gr = GameResult::default();
        let json = serde_json::to_string(&gr).unwrap();
        let back: GameResult = serde_json::from_str(&json).unwrap();
        assert_eq!(gr.home.score, back.home.score);
    }
}
