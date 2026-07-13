use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::enums::{PlayerAction, ReRollSource, SkillCategory, SkillId, Weather};
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
    Move {
        player_id: PlayerId,
        squares: Vec<FieldCoordinate>,
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
    /// Java: `DialogReRollForTargetsParameter` (via `AbstractStepModifierMultipleBlock.
    /// decideNextStep`/`createDialogParameter`) — lets the coach pick which still-failing
    /// multi-block target to re-roll, and with what source, for Dauntless-multi/
    /// FoulAppearance-multi.
    ReRollForTargets {
        player_id: PlayerId,
        target_ids: Vec<String>,
        minimum_rolls: HashMap<String, i32>,
        re_rolled_action: String,
        re_roll_available_against: Vec<String>,
        pro_re_roll_available: bool,
        team_re_roll_available: bool,
        consummate_available: bool,
        re_roll_skill: Option<SkillId>,
        single_use_re_roll_source: Option<ReRollSource>,
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
        /// Java: `DialogPlayerChoiceParameter.descriptions` — a flat list of explanatory
        /// tooltip strings for the dialog as a whole (NOT parallel/indexed to
        /// `eligible_players` — Java's own call sites pass 0 or 1 entries regardless of how
        /// many eligible players there are). Empty when the Java call site passes `null`.
        descriptions: Vec<String>,
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
    /// Java: DialogBloodlustActionParameter — vampire failed blood lust; offer action change.
    BloodlustAction {
        player_id: PlayerId,
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

    #[test]
    fn serde_block_choice_prompt() {
        let p = AgentPrompt::BlockChoice {
            attacker_id: "att".into(), defender_id: "def".into(),
            dice: vec![1, 3, 5], own_choice: true, nr_of_dice: 2,
        };
        let json = serde_json::to_string(&p).unwrap();
        let back: AgentPrompt = serde_json::from_str(&json).unwrap();
        assert_eq!(p, back);
    }

    #[test]
    fn serde_select_skill_prompt() {
        use crate::enums::SkillCategory;
        let p = AgentPrompt::SelectSkill {
            player_id: "p1".into(),
            available: vec![(SkillCategory::General, vec![1, 2, 3])],
        };
        let json = serde_json::to_string(&p).unwrap();
        let back: AgentPrompt = serde_json::from_str(&json).unwrap();
        assert_eq!(p, back);
    }

    #[test]
    fn serde_pushback_prompt() {
        let p = AgentPrompt::Pushback {
            attacker_id: "att".into(), defender_id: "def".into(),
            squares: vec![
                FieldCoordinate::new(10, 7),
                FieldCoordinate::new(10, 8),
            ],
        };
        let json = serde_json::to_string(&p).unwrap();
        let back: AgentPrompt = serde_json::from_str(&json).unwrap();
        assert_eq!(p, back);
    }

    #[test]
    fn serde_trickster_move_prompt() {
        let p = AgentPrompt::TricksterMove {
            player_id: "trickster".into(),
            squares: vec![FieldCoordinate::new(5, 5)],
        };
        let json = serde_json::to_string(&p).unwrap();
        let back: AgentPrompt = serde_json::from_str(&json).unwrap();
        assert_eq!(p, back);
    }

    #[test]
    fn serde_activate_player_prompt() {
        use crate::enums::PlayerAction;
        let p = AgentPrompt::ActivatePlayer {
            eligible_players: vec![
                ("p1".into(), vec![PlayerAction::Move, PlayerAction::Block]),
                ("p2".into(), vec![PlayerAction::Move]),
            ],
        };
        let json = serde_json::to_string(&p).unwrap();
        let back: AgentPrompt = serde_json::from_str(&json).unwrap();
        assert_eq!(p, back);
    }

    #[test]
    fn agent_response_select_skill_round_trip() {
        let r = AgentResponse::SelectSkill { skill_id: 42 };
        let json = serde_json::to_string(&r).unwrap();
        let back: AgentResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(r, back);
    }

    #[test]
    fn agent_response_team_setup_round_trip() {
        let r = AgentResponse::TeamSetup {
            placements: vec![("p1".into(), FieldCoordinate::new(5, 7))],
        };
        let json = serde_json::to_string(&r).unwrap();
        let back: AgentResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(r, back);
    }

    #[test]
    fn serde_reroll_for_targets_prompt() {
        let mut minimum_rolls = HashMap::new();
        minimum_rolls.insert("t1".to_string(), 4);
        let p = AgentPrompt::ReRollForTargets {
            player_id: "p1".into(),
            target_ids: vec!["t1".into()],
            minimum_rolls,
            re_rolled_action: "DAUNTLESS".into(),
            re_roll_available_against: vec!["t1".into()],
            pro_re_roll_available: false,
            team_re_roll_available: true,
            consummate_available: false,
            re_roll_skill: None,
            single_use_re_roll_source: Some(ReRollSource::new("LORD_OF_CHAOS")),
        };
        let json = serde_json::to_string(&p).unwrap();
        let back: AgentPrompt = serde_json::from_str(&json).unwrap();
        assert_eq!(p, back);
    }
}
