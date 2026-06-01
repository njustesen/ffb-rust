use serde::{Deserialize, Serialize};
use crate::enums::{PlayerAction, ReRollSource, SkillCategory, Weather};
use crate::model::player::PlayerId;
use crate::types::FieldCoordinate;

/// The server's request for an agent decision (replaces 70+ Java Dialog*Parameter classes).
///
/// The engine emits one of these whenever it needs an agent choice.
/// The agent calls `apply()` with the matching `AgentResponse`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum AgentPrompt {
    // ── Block ──────────────────────────────────────────────────────────────────
    BlockChoice {
        attacker_id: PlayerId,
        defender_id: PlayerId,
        dice: Vec<i32>,
        own_choice: bool,
        nr_of_dice: i32,
    },
    BlockChoiceProperties {
        attacker_id: PlayerId,
        defender_id: PlayerId,
        can_reroll: bool,
        reroll_sources: Vec<ReRollSource>,
    },

    // ── Movement ───────────────────────────────────────────────────────────────
    ActivatePlayer {
        eligible_players: Vec<(PlayerId, Vec<PlayerAction>)>,
    },
    FollowUp {
        attacker_id: PlayerId,
        target_coord: FieldCoordinate,
    },
    HitAndRun {
        player_id: PlayerId,
        squares: Vec<FieldCoordinate>,
    },
    TricksterMove {
        player_id: PlayerId,
        squares: Vec<FieldCoordinate>,
    },
    Pushback {
        attacker_id: PlayerId,
        defender_id: PlayerId,
        squares: Vec<FieldCoordinate>,
    },

    // ── Re-rolls ───────────────────────────────────────────────────────────────
    ReRollOffer {
        source: ReRollSource,
        action: String,
        team_id: String,
    },

    // ── Skill use ──────────────────────────────────────────────────────────────
    SkillUse {
        player_id: PlayerId,
        skill_id: u16,
        skill_name: String,
    },
    PilingOn {
        player_id: PlayerId,
        target_id: PlayerId,
    },
    DefenderAction {
        player_id: PlayerId,
        actions: Vec<String>,
    },

    // ── Interception / catch ───────────────────────────────────────────────────
    Interception {
        player_id: PlayerId,
        target_number: i32,
    },

    // ── Apothecary ─────────────────────────────────────────────────────────────
    ApothecaryChoice {
        player_id: PlayerId,
        can_heal: bool,
    },
    UseApothecary {
        player_id: PlayerId,
        apothecary_type: String,
    },

    // ── Team setup ─────────────────────────────────────────────────────────────
    TeamSetup {
        team_id: String,
        players: Vec<PlayerId>,
    },
    SetupError {
        team_id: String,
        error: String,
    },

    // ── Kickoff / game flow ────────────────────────────────────────────────────
    CoinChoice { is_home: bool },
    ReceiveChoice { team_id: String },
    Touchback { eligible_players: Vec<(PlayerId, FieldCoordinate)> },
    KickoffReturn { eligible_players: Vec<PlayerId> },
    KickBall,

    // ── Inducements ────────────────────────────────────────────────────────────
    BuyInducements {
        team_id: String,
        available: Vec<(String, i32)>,
        budget: i32,
    },
    BuyPrayersAndInducements {
        team_id: String,
        available: Vec<(String, i32)>,
        prayers: Vec<(String, i32)>,
        budget: i32,
    },
    PettyCash {
        team_id: String,
        amount: i32,
    },
    UseInducement {
        team_id: String,
        inducement_id: String,
    },
    WizardSpell {
        team_id: String,
        target_coord: Option<FieldCoordinate>,
    },
    Journeymen {
        team_id: String,
        count: i32,
    },

    // ── Misc ───────────────────────────────────────────────────────────────────
    PlayerChoice {
        eligible_players: Vec<PlayerId>,
        reason: String,
    },
    SelectPosition {
        available_positions: Vec<String>,
    },
    SelectSkill {
        player_id: PlayerId,
        available: Vec<(SkillCategory, Vec<u16>)>,
    },
    SelectWeather {
        options: Vec<Weather>,
    },
    ArgueTheCall {
        player_id: PlayerId,
    },
    BriberyAndCorruption {
        team_id: String,
    },
    ConcedeGame {
        team_id: String,
    },
    ConfirmEndAction {
        team_id: String,
    },
    InformationOkay {
        message: String,
    },
    SwarmingPlayers {
        team_id: String,
        eligible_players: Vec<PlayerId>,
    },
    StartGame,
    GameStatistics,
}

/// An agent's response to a prompt.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum AgentResponse {
    BlockChoice { index: i32 },
    UseReRoll { use_reroll: bool },
    UseSkill { use_skill: bool },
    FollowUp { follow_up: bool },
    Pushback { coord: FieldCoordinate },
    ActivatePlayer { player_id: PlayerId, action: PlayerAction },
    ReceiveChoice { receive: bool },
    CoinChoice { heads: bool },
    Touchback { player_id: PlayerId },
    KickBall { coord: FieldCoordinate },
    PlayerChoice { player_id: PlayerId },
    TeamSetup { placements: Vec<(PlayerId, FieldCoordinate)> },
    ApothecaryChoice { heal: bool },
    UseBribe { use_bribe: bool },
    BuyInducements { purchases: Vec<(String, i32)> },
    SelectSkill { skill_id: u16 },
    Confirm,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_coin_choice() {
        let p = AgentPrompt::CoinChoice { is_home: true };
        let json = serde_json::to_string(&p).unwrap();
        let back: AgentPrompt = serde_json::from_str(&json).unwrap();
        assert_eq!(p, back);
    }

    #[test]
    fn serde_agent_response() {
        let r = AgentResponse::UseReRoll { use_reroll: true };
        let json = serde_json::to_string(&r).unwrap();
        let back: AgentResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(r, back);
    }
}
