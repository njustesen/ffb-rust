use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::enums::{SendToBoxReason, SeriousInjuryKind, PlayerState, PS_BADLY_HURT, PS_SERIOUS_INJURY, PS_RIP};
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
    /// Java: PlayerResult.fRushing — rushing yards accumulated (ball-carrying moves).
    pub rushing: i32,
    /// Java: fPassing — passing yards accumulated.
    pub passing: i32,
    /// Java: fPlayerAwards — tally of awards (e.g. MVP nominations).
    pub player_awards: i32,
    /// Java: fTurnsPlayed — number of drive turns in which this player was active.
    pub turns_played: i32,
    /// Java: fCurrentSpps — the player's SPP total at game start (used for MVP tie-breaking).
    pub current_spps: i32,
    /// Java: fSeriousInjury — serious injury received this game (if any).
    pub serious_injury: Option<SeriousInjuryKind>,
    /// Java: fSeriousInjuryDecay — niggling decay applied this game (if any).
    pub serious_injury_decay: Option<SeriousInjuryKind>,
    /// Java: fSendToBoxReason — why this player was sent to the injury box this drive.
    pub send_to_box_reason: Option<SendToBoxReason>,
    /// Java: fSendToBoxTurn — turn on which the player was sent to the box.
    pub send_to_box_turn: i32,
    /// Java: fSendToBoxHalf — half in which the player was sent to the box.
    pub send_to_box_half: i32,
    /// Java: fSendToBoxByPlayerId — id of the player that caused the send-to-box.
    pub send_to_box_by_player_id: Option<String>,
    /// Java: fHasUsedSecretWeapon — whether this player fired a secret weapon this drive.
    pub has_used_secret_weapon: bool,
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
    /// Java: `TeamResult.fPettyCashTransferred` — amount of petty cash the team chose to spend.
    pub petty_cash_transferred: i32,
    /// Java: `TeamResult.fPettyCashUsed` — petty cash actually spent on inducements.
    pub petty_cash_used: i32,
    /// Java: `TeamResult.fSpectators` — spectators attending this game.
    pub spectators: i32,
    /// Java: `TeamResult.fSpirallingExpenses` — spiralling expenses penalty applied.
    pub spiralling_expenses: i32,
    /// Java: `TeamResult.fBadlyHurtSuffered` — badly hurt casualties count.
    pub badly_hurt_suffered: i32,
    /// Java: `TeamResult.fSeriousInjurySuffered` — serious injury casualties count.
    pub serious_injury_suffered: i32,
    /// Java: `TeamResult.fRipSuffered` — RIP casualties count.
    pub rip_suffered: i32,
    /// Java: `TeamResult.treasurySpentOnInducements` — gold spent on inducements this game.
    pub treasury_spent_on_inducements: i32,
}

impl TeamResult {
    /// Java: `TeamResult.sufferInjury(PlayerState)` — increments the appropriate injury counter.
    pub fn suffer_injury(&mut self, player_state: PlayerState) {
        match player_state.base() {
            PS_BADLY_HURT => self.badly_hurt_suffered += 1,
            PS_SERIOUS_INJURY => self.serious_injury_suffered += 1,
            PS_RIP => self.rip_suffered += 1,
            _ => {}
        }
    }

    /// Java: `GameResult.getPlayerResult(player)` — returns (creating if needed) the player's result.
    pub fn player_result_mut(&mut self, player_id: &str) -> &mut PlayerResult {
        self.player_results.entry(player_id.to_string()).or_default()
    }

    /// Java: `GameResult.getPlayerResult(player)` — read-only access to a player's result.
    pub fn player_result(&self, player_id: &str) -> Option<&PlayerResult> {
        self.player_results.get(player_id)
    }
}

/// Full game result.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GameResult {
    pub home: TeamResult,
    pub away: TeamResult,
}

impl GameResult {
    /// Java: `GameResult.getTeamResult(team)` — returns the TeamResult for the given team id.
    pub fn team_result_mut(&mut self, is_home: bool) -> &mut TeamResult {
        if is_home { &mut self.home } else { &mut self.away }
    }

    pub fn team_result(&self, is_home: bool) -> &TeamResult {
        if is_home { &self.home } else { &self.away }
    }

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
    fn suffer_injury_badly_hurt() {
        let mut tr = TeamResult::default();
        tr.suffer_injury(PlayerState::new(PS_BADLY_HURT));
        assert_eq!(tr.badly_hurt_suffered, 1);
        assert_eq!(tr.serious_injury_suffered, 0);
        assert_eq!(tr.rip_suffered, 0);
    }

    #[test]
    fn suffer_injury_serious_injury() {
        let mut tr = TeamResult::default();
        tr.suffer_injury(PlayerState::new(PS_SERIOUS_INJURY));
        assert_eq!(tr.serious_injury_suffered, 1);
    }

    #[test]
    fn suffer_injury_rip() {
        let mut tr = TeamResult::default();
        tr.suffer_injury(PlayerState::new(PS_RIP));
        assert_eq!(tr.rip_suffered, 1);
    }

    #[test]
    fn suffer_injury_stunned_is_noop() {
        use crate::enums::PS_STUNNED;
        let mut tr = TeamResult::default();
        tr.suffer_injury(PlayerState::new(PS_STUNNED));
        assert_eq!(tr.badly_hurt_suffered, 0);
        assert_eq!(tr.serious_injury_suffered, 0);
        assert_eq!(tr.rip_suffered, 0);
    }

    #[test]
    fn player_result_mut_creates_on_first_access() {
        let mut tr = TeamResult::default();
        {
            let pr = tr.player_result_mut("p1");
            pr.touchdowns = 2;
        }
        assert_eq!(tr.player_results["p1"].touchdowns, 2);
    }

    #[test]
    fn player_result_returns_none_for_unknown() {
        let tr = TeamResult::default();
        assert!(tr.player_result("nobody").is_none());
    }

    #[test]
    fn team_result_mut_selects_correct_side() {
        let mut gr = GameResult::default();
        gr.team_result_mut(true).score = 3;
        gr.team_result_mut(false).score = 1;
        assert_eq!(gr.home.score, 3);
        assert_eq!(gr.away.score, 1);
    }

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
        assert_eq!(pr.passing, 0);
        assert_eq!(pr.player_awards, 0);
        assert_eq!(pr.turns_played, 0);
        assert_eq!(pr.current_spps, 0);
        assert!(pr.serious_injury.is_none());
        assert!(pr.send_to_box_reason.is_none());
        assert!(!pr.has_used_secret_weapon);
    }

    #[test]
    fn team_result_injury_fields_default_zero() {
        let tr = TeamResult::default();
        assert_eq!(tr.spectators, 0);
        assert_eq!(tr.badly_hurt_suffered, 0);
        assert_eq!(tr.serious_injury_suffered, 0);
        assert_eq!(tr.rip_suffered, 0);
        assert_eq!(tr.spiralling_expenses, 0);
        assert_eq!(tr.treasury_spent_on_inducements, 0);
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
