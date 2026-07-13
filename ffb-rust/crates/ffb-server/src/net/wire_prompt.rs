/// Outgoing wire-format structs for the *other* direction: engine `AgentPrompt`
/// → client dialog request.
///
/// `wire.rs` covers `GameEvent → WireReport` (state changes the client should
/// display as a log entry).  This file covers the complementary gap: the
/// engine's "please show this dialog and wait for a choice" signal
/// (`AgentPrompt`) currently has NO outgoing wire encoding anywhere in
/// `ffb-server` — a connected client can never actually be asked to make a
/// choice without this.
///
/// The real Java desktop client expects a `ServerCommandSetDialogParameter`
/// wrapping one of ~70 `Dialog*Parameter` classes, each with its own field
/// set, tagged by a dialog id.  The `ffb-java` reference tree is not present
/// in this checkout, so the exact Java field names/tag values below are a
/// best-effort design that mirrors the established `WireReport` convention
/// in `wire.rs` (SCREAMING_SNAKE_CASE tag via `#[serde(tag = "dialogId")]`,
/// camelCase field renames) rather than a byte-for-byte port of the Java
/// source. If/when `ffb-java/.../net/commands/ServerCommandSetDialogParameter*.java`
/// becomes available, this should be reconciled against it.
use serde::Serialize;
use ffb_model::prompts::AgentPrompt;
use ffb_model::types::FieldCoordinate;

/// Java: `ServerCommandSetDialogParameter` → JSON `{ "netCommandId": "serverSetDialogParameter", "dialogParameter": {...} }`
#[derive(Serialize)]
pub struct OutgoingSetDialogParameter {
    #[serde(rename = "netCommandId")]
    pub net_command_id: &'static str,
    #[serde(rename = "dialogParameter")]
    pub dialog_parameter: WireDialog,
}

impl OutgoingSetDialogParameter {
    pub fn new(dialog_parameter: WireDialog) -> Self {
        Self { net_command_id: "serverSetDialogParameter", dialog_parameter }
    }
}

/// A player id paired with the actions available to them — used by
/// `AgentPrompt::ActivatePlayer`.
#[derive(Serialize)]
pub struct WireEligibleActions {
    #[serde(rename = "playerId")]
    pub player_id: String,
    pub actions: Vec<String>,
}

/// A player id paired with a target coordinate — used by `AgentPrompt::Touchback`.
#[derive(Serialize)]
pub struct WirePlayerCoord {
    #[serde(rename = "playerId")]
    pub player_id: String,
    pub coord: FieldCoordinate,
}

/// A purchasable item name paired with its cost — used by inducement/prayer dialogs.
#[derive(Serialize)]
pub struct WirePriced {
    pub name: String,
    pub cost: i32,
}

/// A skill category paired with the skill ids available in it — used by `SelectSkill`.
#[derive(Serialize)]
pub struct WireSkillCategoryChoice {
    pub category: String,
    #[serde(rename = "skillIds")]
    pub skill_ids: Vec<u16>,
}

/// Java: `Dialog*Parameter` implementations → tagged by `"dialogId"` field.
///
/// Variant names are SCREAMING_SNAKE_CASE, matching the `WireReport`
/// convention used for the report direction.
#[derive(Serialize)]
#[serde(tag = "dialogId")]
pub enum WireDialog {
    #[serde(rename = "BLOCK_CHOICE")]
    BlockChoice {
        #[serde(rename = "attackerId")] attacker_id: String,
        #[serde(rename = "defenderId")] defender_id: String,
        dice: Vec<i32>,
        #[serde(rename = "ownChoice")] own_choice: bool,
        #[serde(rename = "nrOfDice")] nr_of_dice: i32,
    },

    #[serde(rename = "BLOCK_CHOICE_PROPERTIES")]
    BlockChoiceProperties {
        #[serde(rename = "attackerId")] attacker_id: String,
        #[serde(rename = "defenderId")] defender_id: String,
        #[serde(rename = "canReroll")] can_reroll: bool,
        #[serde(rename = "rerollSources")] reroll_sources: Vec<String>,
    },

    #[serde(rename = "ACTIVATE_PLAYER")]
    ActivatePlayer { #[serde(rename = "eligiblePlayers")] eligible_players: Vec<WireEligibleActions> },

    #[serde(rename = "MOVE")]
    Move { #[serde(rename = "playerId")] player_id: String, squares: Vec<FieldCoordinate> },

    #[serde(rename = "FOLLOW_UP")]
    FollowUp {
        #[serde(rename = "attackerId")] attacker_id: String,
        #[serde(rename = "targetCoord")] target_coord: FieldCoordinate,
    },

    #[serde(rename = "HIT_AND_RUN")]
    HitAndRun { #[serde(rename = "playerId")] player_id: String, squares: Vec<FieldCoordinate> },

    #[serde(rename = "TRICKSTER_MOVE")]
    TricksterMove { #[serde(rename = "playerId")] player_id: String, squares: Vec<FieldCoordinate> },

    #[serde(rename = "PUSHBACK")]
    Pushback {
        #[serde(rename = "attackerId")] attacker_id: String,
        #[serde(rename = "defenderId")] defender_id: String,
        squares: Vec<FieldCoordinate>,
    },

    #[serde(rename = "RE_ROLL_OFFER")]
    ReRollOffer { source: String, action: String, #[serde(rename = "teamId")] team_id: String },

    #[serde(rename = "RE_ROLL_FOR_TARGETS")]
    ReRollForTargets {
        #[serde(rename = "playerId")] player_id: String,
        #[serde(rename = "targetIds")] target_ids: Vec<String>,
        #[serde(rename = "minimumRolls")] minimum_rolls: std::collections::HashMap<String, i32>,
        #[serde(rename = "reRolledAction")] re_rolled_action: String,
        #[serde(rename = "reRollAvailableAgainst")] re_roll_available_against: Vec<String>,
        #[serde(rename = "proReRollAvailable")] pro_re_roll_available: bool,
        #[serde(rename = "teamReRollAvailable")] team_re_roll_available: bool,
        #[serde(rename = "consummateAvailable")] consummate_available: bool,
        #[serde(rename = "reRollSkill")] re_roll_skill: Option<u16>,
        #[serde(rename = "singleUseReRollSource")] single_use_re_roll_source: Option<String>,
    },

    #[serde(rename = "SKILL_USE")]
    SkillUse {
        #[serde(rename = "playerId")] player_id: String,
        #[serde(rename = "skillId")] skill_id: u16,
        #[serde(rename = "skillName")] skill_name: String,
    },

    #[serde(rename = "PILING_ON")]
    PilingOn { #[serde(rename = "playerId")] player_id: String, #[serde(rename = "targetId")] target_id: String },

    #[serde(rename = "DEFENDER_ACTION")]
    DefenderAction { #[serde(rename = "playerId")] player_id: String, actions: Vec<String> },

    #[serde(rename = "INTERCEPTION")]
    Interception { #[serde(rename = "playerId")] player_id: String, #[serde(rename = "targetNumber")] target_number: i32 },

    #[serde(rename = "APOTHECARY_CHOICE")]
    ApothecaryChoice { #[serde(rename = "playerId")] player_id: String, #[serde(rename = "canHeal")] can_heal: bool },

    #[serde(rename = "USE_APOTHECARY")]
    UseApothecary { #[serde(rename = "playerId")] player_id: String, #[serde(rename = "apothecaryType")] apothecary_type: String },

    #[serde(rename = "TEAM_SETUP")]
    TeamSetup { #[serde(rename = "teamId")] team_id: String, players: Vec<String> },

    #[serde(rename = "SETUP_ERROR")]
    SetupError { #[serde(rename = "teamId")] team_id: String, error: String },

    #[serde(rename = "COIN_CHOICE")]
    CoinChoice { #[serde(rename = "isHome")] is_home: bool },

    #[serde(rename = "RECEIVE_CHOICE")]
    ReceiveChoice { #[serde(rename = "teamId")] team_id: String },

    #[serde(rename = "TOUCHBACK")]
    Touchback { #[serde(rename = "eligiblePlayers")] eligible_players: Vec<WirePlayerCoord> },

    #[serde(rename = "KICKOFF_RETURN")]
    KickoffReturn { #[serde(rename = "eligiblePlayers")] eligible_players: Vec<String> },

    #[serde(rename = "KICK_BALL")]
    KickBall,

    #[serde(rename = "BUY_INDUCEMENTS")]
    BuyInducements {
        #[serde(rename = "teamId")] team_id: String,
        available: Vec<WirePriced>,
        budget: i32,
    },

    #[serde(rename = "BUY_PRAYERS_AND_INDUCEMENTS")]
    BuyPrayersAndInducements {
        #[serde(rename = "teamId")] team_id: String,
        available: Vec<WirePriced>,
        prayers: Vec<WirePriced>,
        budget: i32,
    },

    #[serde(rename = "PETTY_CASH")]
    PettyCash { #[serde(rename = "teamId")] team_id: String, amount: i32 },

    #[serde(rename = "USE_INDUCEMENT")]
    UseInducement { #[serde(rename = "teamId")] team_id: String, #[serde(rename = "inducementId")] inducement_id: String },

    #[serde(rename = "WIZARD_SPELL")]
    WizardSpell { #[serde(rename = "teamId")] team_id: String, #[serde(rename = "targetCoord")] target_coord: Option<FieldCoordinate> },

    #[serde(rename = "JOURNEYMEN")]
    Journeymen { #[serde(rename = "teamId")] team_id: String, count: i32 },

    #[serde(rename = "PLAYER_CHOICE")]
    PlayerChoice { #[serde(rename = "eligiblePlayers")] eligible_players: Vec<String>, reason: String },

    #[serde(rename = "SELECT_POSITION")]
    SelectPosition { #[serde(rename = "availablePositions")] available_positions: Vec<String> },

    #[serde(rename = "SELECT_SKILL")]
    SelectSkill { #[serde(rename = "playerId")] player_id: String, available: Vec<WireSkillCategoryChoice> },

    #[serde(rename = "SELECT_WEATHER")]
    SelectWeather { options: Vec<String> },

    #[serde(rename = "ARGUE_THE_CALL")]
    ArgueTheCall { #[serde(rename = "playerId")] player_id: String },

    #[serde(rename = "BRIBERY_AND_CORRUPTION")]
    BriberyAndCorruption { #[serde(rename = "teamId")] team_id: String },

    #[serde(rename = "CONCEDE_GAME")]
    ConcedeGame { #[serde(rename = "teamId")] team_id: String },

    #[serde(rename = "CONFIRM_END_ACTION")]
    ConfirmEndAction { #[serde(rename = "teamId")] team_id: String },

    #[serde(rename = "INFORMATION_OKAY")]
    InformationOkay { message: String },

    #[serde(rename = "BLOODLUST_ACTION")]
    BloodlustAction { #[serde(rename = "playerId")] player_id: String },

    #[serde(rename = "SWARMING_PLAYERS")]
    SwarmingPlayers { #[serde(rename = "teamId")] team_id: String, #[serde(rename = "eligiblePlayers")] eligible_players: Vec<String> },

    #[serde(rename = "START_GAME")]
    StartGame,

    #[serde(rename = "GAME_STATISTICS")]
    GameStatistics,
}

/// Convert a single `AgentPrompt` to a `WireDialog`.
///
/// Every `AgentPrompt` variant as of Phase ZV has a corresponding `WireDialog`
/// variant, so this always returns `Some`. The signature stays `Option` to
/// match `event_to_report`'s shape and leave room for future prompt variants
/// that may be purely internal (never shown to a client).
pub fn prompt_to_wire(prompt: &AgentPrompt) -> Option<WireDialog> {
    match prompt {
        AgentPrompt::BlockChoice { attacker_id, defender_id, dice, own_choice, nr_of_dice } =>
            Some(WireDialog::BlockChoice {
                attacker_id: attacker_id.clone(), defender_id: defender_id.clone(),
                dice: dice.clone(), own_choice: *own_choice, nr_of_dice: *nr_of_dice,
            }),
        AgentPrompt::BlockChoiceProperties { attacker_id, defender_id, can_reroll, reroll_sources } =>
            Some(WireDialog::BlockChoiceProperties {
                attacker_id: attacker_id.clone(), defender_id: defender_id.clone(),
                can_reroll: *can_reroll,
                reroll_sources: reroll_sources.iter().map(|s| format!("{:?}", s)).collect(),
            }),
        AgentPrompt::ActivatePlayer { eligible_players } =>
            Some(WireDialog::ActivatePlayer {
                eligible_players: eligible_players.iter().map(|(id, actions)| WireEligibleActions {
                    player_id: id.clone(),
                    actions: actions.iter().map(|a| format!("{:?}", a)).collect(),
                }).collect(),
            }),
        AgentPrompt::Move { player_id, squares } =>
            Some(WireDialog::Move { player_id: player_id.clone(), squares: squares.clone() }),
        AgentPrompt::FollowUp { attacker_id, target_coord } =>
            Some(WireDialog::FollowUp { attacker_id: attacker_id.clone(), target_coord: *target_coord }),
        AgentPrompt::HitAndRun { player_id, squares } =>
            Some(WireDialog::HitAndRun { player_id: player_id.clone(), squares: squares.clone() }),
        AgentPrompt::TricksterMove { player_id, squares } =>
            Some(WireDialog::TricksterMove { player_id: player_id.clone(), squares: squares.clone() }),
        AgentPrompt::Pushback { attacker_id, defender_id, squares } =>
            Some(WireDialog::Pushback { attacker_id: attacker_id.clone(), defender_id: defender_id.clone(), squares: squares.clone() }),
        AgentPrompt::ReRollOffer { source, action, team_id } =>
            Some(WireDialog::ReRollOffer { source: source.name.clone(), action: action.clone(), team_id: team_id.clone() }),
        AgentPrompt::ReRollForTargets {
            player_id, target_ids, minimum_rolls, re_rolled_action, re_roll_available_against,
            pro_re_roll_available, team_re_roll_available, consummate_available, re_roll_skill,
            single_use_re_roll_source,
        } => Some(WireDialog::ReRollForTargets {
            player_id: player_id.clone(),
            target_ids: target_ids.clone(),
            minimum_rolls: minimum_rolls.clone(),
            re_rolled_action: re_rolled_action.clone(),
            re_roll_available_against: re_roll_available_against.clone(),
            pro_re_roll_available: *pro_re_roll_available,
            team_re_roll_available: *team_re_roll_available,
            consummate_available: *consummate_available,
            re_roll_skill: re_roll_skill.map(|id| id as u16),
            single_use_re_roll_source: single_use_re_roll_source.as_ref().map(|s| s.name.clone()),
        }),
        AgentPrompt::SkillUse { player_id, skill_id, skill_name } =>
            Some(WireDialog::SkillUse { player_id: player_id.clone(), skill_id: *skill_id, skill_name: skill_name.clone() }),
        AgentPrompt::PilingOn { player_id, target_id } =>
            Some(WireDialog::PilingOn { player_id: player_id.clone(), target_id: target_id.clone() }),
        AgentPrompt::DefenderAction { player_id, actions } =>
            Some(WireDialog::DefenderAction { player_id: player_id.clone(), actions: actions.clone() }),
        AgentPrompt::Interception { player_id, target_number } =>
            Some(WireDialog::Interception { player_id: player_id.clone(), target_number: *target_number }),
        AgentPrompt::ApothecaryChoice { player_id, can_heal } =>
            Some(WireDialog::ApothecaryChoice { player_id: player_id.clone(), can_heal: *can_heal }),
        AgentPrompt::UseApothecary { player_id, apothecary_type } =>
            Some(WireDialog::UseApothecary { player_id: player_id.clone(), apothecary_type: apothecary_type.clone() }),
        AgentPrompt::TeamSetup { team_id, players } =>
            Some(WireDialog::TeamSetup { team_id: team_id.clone(), players: players.clone() }),
        AgentPrompt::SetupError { team_id, error } =>
            Some(WireDialog::SetupError { team_id: team_id.clone(), error: error.clone() }),
        AgentPrompt::CoinChoice { is_home } =>
            Some(WireDialog::CoinChoice { is_home: *is_home }),
        AgentPrompt::ReceiveChoice { team_id } =>
            Some(WireDialog::ReceiveChoice { team_id: team_id.clone() }),
        AgentPrompt::Touchback { eligible_players } =>
            Some(WireDialog::Touchback {
                eligible_players: eligible_players.iter().map(|(id, coord)| WirePlayerCoord {
                    player_id: id.clone(), coord: *coord,
                }).collect(),
            }),
        AgentPrompt::KickoffReturn { eligible_players } =>
            Some(WireDialog::KickoffReturn { eligible_players: eligible_players.clone() }),
        AgentPrompt::KickBall =>
            Some(WireDialog::KickBall),
        AgentPrompt::BuyInducements { team_id, available, budget } =>
            Some(WireDialog::BuyInducements {
                team_id: team_id.clone(),
                available: available.iter().map(|(name, cost)| WirePriced { name: name.clone(), cost: *cost }).collect(),
                budget: *budget,
            }),
        AgentPrompt::BuyPrayersAndInducements { team_id, available, prayers, budget } =>
            Some(WireDialog::BuyPrayersAndInducements {
                team_id: team_id.clone(),
                available: available.iter().map(|(name, cost)| WirePriced { name: name.clone(), cost: *cost }).collect(),
                prayers: prayers.iter().map(|(name, cost)| WirePriced { name: name.clone(), cost: *cost }).collect(),
                budget: *budget,
            }),
        AgentPrompt::PettyCash { team_id, amount } =>
            Some(WireDialog::PettyCash { team_id: team_id.clone(), amount: *amount }),
        AgentPrompt::UseInducement { team_id, inducement_id } =>
            Some(WireDialog::UseInducement { team_id: team_id.clone(), inducement_id: inducement_id.clone() }),
        AgentPrompt::WizardSpell { team_id, target_coord } =>
            Some(WireDialog::WizardSpell { team_id: team_id.clone(), target_coord: *target_coord }),
        AgentPrompt::Journeymen { team_id, count } =>
            Some(WireDialog::Journeymen { team_id: team_id.clone(), count: *count }),
        AgentPrompt::PlayerChoice { eligible_players, reason } =>
            Some(WireDialog::PlayerChoice { eligible_players: eligible_players.clone(), reason: reason.clone() }),
        AgentPrompt::SelectPosition { available_positions } =>
            Some(WireDialog::SelectPosition { available_positions: available_positions.clone() }),
        AgentPrompt::SelectSkill { player_id, available } =>
            Some(WireDialog::SelectSkill {
                player_id: player_id.clone(),
                available: available.iter().map(|(cat, ids)| WireSkillCategoryChoice {
                    category: format!("{:?}", cat), skill_ids: ids.clone(),
                }).collect(),
            }),
        AgentPrompt::SelectWeather { options } =>
            Some(WireDialog::SelectWeather { options: options.iter().map(|w| format!("{:?}", w)).collect() }),
        AgentPrompt::ArgueTheCall { player_id } =>
            Some(WireDialog::ArgueTheCall { player_id: player_id.clone() }),
        AgentPrompt::BriberyAndCorruption { team_id } =>
            Some(WireDialog::BriberyAndCorruption { team_id: team_id.clone() }),
        AgentPrompt::ConcedeGame { team_id } =>
            Some(WireDialog::ConcedeGame { team_id: team_id.clone() }),
        AgentPrompt::ConfirmEndAction { team_id } =>
            Some(WireDialog::ConfirmEndAction { team_id: team_id.clone() }),
        AgentPrompt::InformationOkay { message } =>
            Some(WireDialog::InformationOkay { message: message.clone() }),
        AgentPrompt::BloodlustAction { player_id } =>
            Some(WireDialog::BloodlustAction { player_id: player_id.clone() }),
        AgentPrompt::SwarmingPlayers { team_id, eligible_players } =>
            Some(WireDialog::SwarmingPlayers { team_id: team_id.clone(), eligible_players: eligible_players.clone() }),
        AgentPrompt::StartGame =>
            Some(WireDialog::StartGame),
        AgentPrompt::GameStatistics =>
            Some(WireDialog::GameStatistics),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{PlayerAction, ReRollSource, SkillCategory, Weather};

    fn jsn(dialog: &WireDialog) -> String { serde_json::to_string(dialog).unwrap() }

    #[test]
    fn outgoing_set_dialog_parameter_has_net_command_id() {
        let json = serde_json::to_string(&OutgoingSetDialogParameter::new(WireDialog::KickBall)).unwrap();
        assert!(json.contains("\"netCommandId\":\"serverSetDialogParameter\""));
        assert!(json.contains("\"dialogParameter\""));
        assert!(json.contains("\"dialogId\":\"KICK_BALL\""));
    }

    #[test]
    fn block_choice_prompt_converts() {
        let prompt = AgentPrompt::BlockChoice {
            attacker_id: "a".into(), defender_id: "d".into(),
            dice: vec![1, 3, 5], own_choice: true, nr_of_dice: 2,
        };
        let json = jsn(&prompt_to_wire(&prompt).unwrap());
        assert!(json.contains("\"dialogId\":\"BLOCK_CHOICE\""));
        assert!(json.contains("\"nrOfDice\":2"));
    }

    #[test]
    fn block_choice_properties_prompt_converts() {
        let prompt = AgentPrompt::BlockChoiceProperties {
            attacker_id: "a".into(), defender_id: "d".into(),
            can_reroll: true, reroll_sources: vec![ReRollSource::new("teamReRoll")],
        };
        let json = jsn(&prompt_to_wire(&prompt).unwrap());
        assert!(json.contains("\"dialogId\":\"BLOCK_CHOICE_PROPERTIES\""));
        assert!(json.contains("\"canReroll\":true"));
    }

    #[test]
    fn activate_player_prompt_converts() {
        let prompt = AgentPrompt::ActivatePlayer {
            eligible_players: vec![("p1".into(), vec![PlayerAction::Move, PlayerAction::Block])],
        };
        let json = jsn(&prompt_to_wire(&prompt).unwrap());
        assert!(json.contains("\"dialogId\":\"ACTIVATE_PLAYER\""));
        assert!(json.contains("\"playerId\":\"p1\""));
    }

    #[test]
    fn move_prompt_converts() {
        let prompt = AgentPrompt::Move { player_id: "p".into(), squares: vec![FieldCoordinate::new(3, 4)] };
        let json = jsn(&prompt_to_wire(&prompt).unwrap());
        assert!(json.contains("\"dialogId\":\"MOVE\""));
        assert!(json.contains("\"x\":3"));
    }

    #[test]
    fn coin_choice_prompt_converts() {
        let prompt = AgentPrompt::CoinChoice { is_home: true };
        let json = jsn(&prompt_to_wire(&prompt).unwrap());
        assert!(json.contains("\"dialogId\":\"COIN_CHOICE\""));
        assert!(json.contains("\"isHome\":true"));
    }

    #[test]
    fn receive_choice_prompt_converts() {
        let prompt = AgentPrompt::ReceiveChoice { team_id: "home".into() };
        let json = jsn(&prompt_to_wire(&prompt).unwrap());
        assert!(json.contains("\"dialogId\":\"RECEIVE_CHOICE\""));
    }

    #[test]
    fn touchback_prompt_converts() {
        let prompt = AgentPrompt::Touchback { eligible_players: vec![("p1".into(), FieldCoordinate::new(1, 1))] };
        let json = jsn(&prompt_to_wire(&prompt).unwrap());
        assert!(json.contains("\"dialogId\":\"TOUCHBACK\""));
    }

    #[test]
    fn buy_inducements_prompt_converts() {
        let prompt = AgentPrompt::BuyInducements {
            team_id: "home".into(), available: vec![("bloodweiser_babe".into(), 50000)], budget: 100000,
        };
        let json = jsn(&prompt_to_wire(&prompt).unwrap());
        assert!(json.contains("\"dialogId\":\"BUY_INDUCEMENTS\""));
        assert!(json.contains("\"cost\":50000"));
    }

    #[test]
    fn select_skill_prompt_converts() {
        let prompt = AgentPrompt::SelectSkill {
            player_id: "p1".into(), available: vec![(SkillCategory::General, vec![1, 2])],
        };
        let json = jsn(&prompt_to_wire(&prompt).unwrap());
        assert!(json.contains("\"dialogId\":\"SELECT_SKILL\""));
        assert!(json.contains("\"skillIds\":[1,2]"));
    }

    #[test]
    fn select_weather_prompt_converts() {
        let prompt = AgentPrompt::SelectWeather { options: vec![Weather::Nice, Weather::VerySunny] };
        let json = jsn(&prompt_to_wire(&prompt).unwrap());
        assert!(json.contains("\"dialogId\":\"SELECT_WEATHER\""));
    }

    #[test]
    fn argue_the_call_prompt_converts() {
        let prompt = AgentPrompt::ArgueTheCall { player_id: "p".into() };
        let json = jsn(&prompt_to_wire(&prompt).unwrap());
        assert!(json.contains("\"dialogId\":\"ARGUE_THE_CALL\""));
    }

    #[test]
    fn start_game_and_game_statistics_are_unit_dialogs() {
        assert!(jsn(&prompt_to_wire(&AgentPrompt::StartGame).unwrap()).contains("\"dialogId\":\"START_GAME\""));
        assert!(jsn(&prompt_to_wire(&AgentPrompt::GameStatistics).unwrap()).contains("\"dialogId\":\"GAME_STATISTICS\""));
    }

    #[test]
    fn swarming_players_prompt_converts() {
        let prompt = AgentPrompt::SwarmingPlayers { team_id: "away".into(), eligible_players: vec!["p1".into(), "p2".into()] };
        let json = jsn(&prompt_to_wire(&prompt).unwrap());
        assert!(json.contains("\"dialogId\":\"SWARMING_PLAYERS\""));
        assert!(json.contains("\"eligiblePlayers\":[\"p1\",\"p2\"]"));
    }

    #[test]
    fn information_okay_prompt_converts() {
        let prompt = AgentPrompt::InformationOkay { message: "hello".into() };
        let json = jsn(&prompt_to_wire(&prompt).unwrap());
        assert!(json.contains("\"dialogId\":\"INFORMATION_OKAY\""));
        assert!(json.contains("\"message\":\"hello\""));
    }
}
