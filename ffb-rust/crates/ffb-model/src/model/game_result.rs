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
    pub landings: i32,
    pub catches_with_additional_spp: i32,
    pub completions_with_additional_spp: i32,
    pub casualties_with_additional_spp: i32,
    pub mvp: bool,
    pub spp_gained: i32,
    pub fouls: i32,
    /// Java: fBlocks — number of blocks thrown this game.
    pub blocks: i32,
    /// Set during end-game player loss check when player defects on illegal concession.
    pub defecting: bool,
}

/// Per-team final game result.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TeamResult {
    pub score: i32,
    pub winnings: i32,
    pub fan_factor_modifier: i32,
    pub dedicated_fans_modifier: i32,
    pub fan_factor: i32,
    pub fame: i32,
    pub player_results: HashMap<PlayerId, PlayerResult>,
    /// Set when this team conceded the game.
    pub conceded: bool,
    /// Set when this team was considered stalling (reduces winnings by 1).
    pub stalled: bool,
    pub raised_dead: i32,
    /// Java: `TeamResult.teamValue` — the team's current TV at game start.
    pub team_value: i32,
    /// Java: `TeamResult.pettyCashFromTvDiff` — petty cash available to the underdog.
    pub petty_cash_from_tv_diff: i32,
    /// Java: `TeamResult.penaltyScore` — penalty-shootout goals scored. Default -1 in Java
    /// (unset); represented as 0 here since Rust uses 0-defaults and -1 has no semantic use.
    pub penalty_score: i32,
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

    #[test]
    fn draw_returns_none_winner() {
        let gr = GameResult::default();
        assert_eq!(gr.winner(), None);
    }

    #[test]
    fn player_result_fields_default_zero() {
        let pr = PlayerResult::default();
        assert_eq!(pr.touchdowns, 0);
        assert_eq!(pr.casualties, 0);
        assert_eq!(pr.spp_gained, 0);
        assert!(!pr.mvp);
    }

    #[test]
    fn team_result_accumulates_player_results() {
        let mut tr = TeamResult::default();
        let mut pr = PlayerResult::default();
        pr.touchdowns = 2;
        pr.spp_gained = 6;
        tr.player_results.insert("p1".to_string(), pr);
        assert_eq!(tr.player_results["p1"].touchdowns, 2);
        assert_eq!(tr.player_results.len(), 1);
    }
}
