/// Outgoing wire-format structs matching the Java FFB server's JSON output.
///
/// The Java client parses these by exact field name.  Field names here
/// intentionally preserve Java's camelCase JSON keys (e.g. `"reportId"`,
/// `"modelChangeList"`).  These are NOT the same as the `ffb_protocol`
/// `ServerCommand` structs, which model the client→server direction.
///
/// Reference: `ffb-common/.../net/commands/ServerCommandModelSync.java`
use serde::Serialize;
use ffb_model::events::GameEvent;

// ── ReportList / WireReport ────────────────────────────────────────────────

/// Java: `ReportList` → JSON `{ "reports": [...] }`
#[derive(Serialize)]
pub struct WireReportList {
    pub reports: Vec<WireReport>,
}

/// Java: `IReport` implementations → tagged by `"reportId"` field.
///
/// Each variant's `rename` must match the Java `ReportId` enum constant name
/// exactly (SCREAMING_SNAKE_CASE), e.g. `"BLOCK"`, `"DODGE_ROLL"`.
#[derive(Serialize)]
#[serde(tag = "reportId")]
pub enum WireReport {
    /// Java: `ReportBlock` — announces a block declaration.
    #[serde(rename = "BLOCK")]
    Block {
        #[serde(rename = "defenderId")]
        defender_id: String,
    },

    /// Java: `ReportBlockRoll` — announces block dice result.
    #[serde(rename = "BLOCK_ROLL")]
    BlockRoll {
        #[serde(rename = "attackerId")]
        attacker_id: String,
        #[serde(rename = "defenderId")]
        defender_id: String,
        #[serde(rename = "nrOfDice")]
        nr_of_dice: i32,
        dice: Vec<i32>,
        #[serde(rename = "selectedDie")]
        selected_die: i32,
        #[serde(rename = "ownChoice")]
        own_choice: bool,
        #[serde(rename = "reRolled")]
        re_rolled: bool,
    },

    /// Java: `ReportDodgeRoll` / `ReportSkillRoll` subclass.
    #[serde(rename = "DODGE_ROLL")]
    DodgeRoll {
        #[serde(rename = "playerId")]
        player_id: String,
        successful: bool,
        roll: i32,
        #[serde(rename = "minimumRoll")]
        minimum_roll: i32,
        #[serde(rename = "reRolled")]
        re_rolled: bool,
    },

    /// Java: `ReportGoForItRoll`
    #[serde(rename = "GO_FOR_IT_ROLL")]
    GoForItRoll {
        #[serde(rename = "playerId")]
        player_id: String,
        successful: bool,
        roll: i32,
        #[serde(rename = "minimumRoll")]
        minimum_roll: i32,
        #[serde(rename = "reRolled")]
        re_rolled: bool,
    },

    /// Java: `ReportPickupRoll`
    #[serde(rename = "PICKUP_ROLL")]
    PickupRoll {
        #[serde(rename = "playerId")]
        player_id: String,
        successful: bool,
        roll: i32,
        #[serde(rename = "minimumRoll")]
        minimum_roll: i32,
        #[serde(rename = "reRolled")]
        re_rolled: bool,
    },

    /// Java: `ReportCatchRoll`
    #[serde(rename = "CATCH_ROLL")]
    CatchRoll {
        #[serde(rename = "playerId")]
        player_id: String,
        successful: bool,
        roll: i32,
        #[serde(rename = "minimumRoll")]
        minimum_roll: i32,
        #[serde(rename = "reRolled")]
        re_rolled: bool,
    },

    /// Java: `ReportPassRoll`
    #[serde(rename = "PASS_ROLL")]
    PassRoll {
        #[serde(rename = "playerId")]
        player_id: String,
        successful: bool,
        roll: i32,
        #[serde(rename = "minimumRoll")]
        minimum_roll: i32,
        #[serde(rename = "reRolled")]
        re_rolled: bool,
        #[serde(rename = "passingDistance")]
        passing_distance: Option<String>,
        #[serde(rename = "passResult")]
        pass_result: String,
        #[serde(rename = "hailMaryPass")]
        hail_mary_pass: bool,
        bomb: bool,
    },

    /// Java: `ReportInjury`
    #[serde(rename = "INJURY")]
    Injury {
        #[serde(rename = "attackerId")]
        attacker_id: Option<String>,
        #[serde(rename = "defenderId")]
        defender_id: String,
        #[serde(rename = "armorRoll")]
        armor_roll: Vec<i32>,
        #[serde(rename = "injuryRoll")]
        injury_roll: Vec<i32>,
        #[serde(rename = "armorBroken")]
        armor_broken: bool,
        injury: Option<u32>,
        #[serde(rename = "casualtyRoll")]
        casualty_roll: Vec<i32>,
        #[serde(rename = "seriousInjury")]
        serious_injury: Option<String>,
    },

    /// Java: `ReportReRoll`
    #[serde(rename = "RE_ROLL")]
    ReRoll {
        #[serde(rename = "teamId")]
        team_id: String,
        source: String,
        #[serde(rename = "reRolledAction")]
        re_rolled_action: Option<String>,
    },

    /// Java: `ReportSkillUse`
    #[serde(rename = "SKILL_USE")]
    SkillUse {
        #[serde(rename = "playerId")]
        player_id: Option<String>,
        skill: String,
        used: bool,
        #[serde(rename = "skillUse")]
        skill_use: String,
    },

    /// Java: `ReportPlayerAction` — player activates with an action type.
    #[serde(rename = "PLAYER_ACTION")]
    PlayerAction {
        #[serde(rename = "playerId")]
        player_id: String,
        #[serde(rename = "playerAction")]
        player_action: String,
    },

    /// Java: `ReportTurnEnd`
    #[serde(rename = "TURN_END")]
    TurnEnd {
        #[serde(rename = "teamId")]
        team_id: String,
        #[serde(rename = "turnNr")]
        turn_nr: i32,
    },

    /// Java: `ReportKickoffResult`
    #[serde(rename = "KICKOFF_RESULT")]
    KickoffResult { result: String },

    /// Java: `ReportCoinThrow`
    #[serde(rename = "COIN_THROW")]
    CoinThrow {
        #[serde(rename = "homeWon")]
        home_won: bool,
    },

    /// Java: `ReportReceiveChoice`
    #[serde(rename = "RECEIVE_CHOICE")]
    ReceiveChoice {
        #[serde(rename = "teamId")]
        team_id: String,
        receive: bool,
    },

    /// Java: `ReportTouchdown`
    #[serde(rename = "TOUCHDOWN")]
    Touchdown {
        #[serde(rename = "playerId")]
        player_id: String,
    },

    /// Java: `ReportFoul`
    #[serde(rename = "FOUL")]
    Foul {
        #[serde(rename = "defenderId")]
        defender_id: String,
    },

    /// Java: `ReportWeatherChange`
    #[serde(rename = "WEATHER_CHANGE")]
    WeatherChange { weather: String },
}

// ── ModelChangeList ────────────────────────────────────────────────────────

/// Java: `ModelChangeList` → JSON `{ "modelChangeArray": [...] }`
#[derive(Serialize)]
pub struct WireModelChangeList {
    #[serde(rename = "modelChangeArray")]
    pub model_change_array: Vec<WireModelChange>,
}

/// Java: `ModelChange` → JSON `{ "modelChangeId": "...", "modelChangeKey": "...", "modelChangeValue": ... }`
#[derive(Serialize)]
pub struct WireModelChange {
    #[serde(rename = "modelChangeId")]
    pub model_change_id: String,
    #[serde(rename = "modelChangeKey")]
    pub model_change_key: String,
    #[serde(rename = "modelChangeValue")]
    pub model_change_value: serde_json::Value,
}

// ── ServerCommandModelSync outgoing ────────────────────────────────────────

/// Outgoing `serverModelSync` command matching Java's wire format exactly.
///
/// Java: `ServerCommandModelSync`
#[derive(Serialize)]
pub struct OutgoingModelSync {
    #[serde(rename = "netCommandId")]
    pub net_command_id: &'static str,
    #[serde(rename = "commandNr")]
    pub command_nr: i64,
    #[serde(rename = "modelChangeList")]
    pub model_change_list: WireModelChangeList,
    #[serde(rename = "reportList")]
    pub report_list: WireReportList,
    pub animation: serde_json::Value,
    pub sound: Option<String>,
    #[serde(rename = "gameTime")]
    pub game_time: i64,
    #[serde(rename = "turnTime")]
    pub turn_time: i64,
}

impl OutgoingModelSync {
    pub fn new(command_nr: i64, reports: Vec<WireReport>) -> Self {
        Self {
            net_command_id: "serverModelSync",
            command_nr,
            model_change_list: WireModelChangeList { model_change_array: vec![] },
            report_list: WireReportList { reports },
            animation: serde_json::Value::Null,
            sound: None,
            game_time: 0,
            turn_time: 0,
        }
    }
}

// ── GameEvent → WireReport conversion ─────────────────────────────────────

/// Convert a single `GameEvent` to a `WireReport` if there is a matching
/// report type.  Returns `None` for events that have no report equivalent
/// (e.g. internal state changes that the client doesn't display).
pub fn event_to_report(event: &GameEvent) -> Option<WireReport> {
    match event {
        GameEvent::BlockRoll { attacker_id, defender_id, nr_of_dice, dice, selected_index, own_choice, rerolled, .. } =>
            Some(WireReport::BlockRoll {
                attacker_id: attacker_id.clone(),
                defender_id: defender_id.clone(),
                nr_of_dice: *nr_of_dice,
                dice: dice.clone(),
                selected_die: *selected_index,
                own_choice: *own_choice,
                re_rolled: *rerolled,
            }),
        GameEvent::Block { defender_id } =>
            Some(WireReport::Block { defender_id: defender_id.clone() }),
        GameEvent::DodgeRoll { player_id, target, roll, success, rerolled } =>
            Some(WireReport::DodgeRoll {
                player_id: player_id.clone(),
                successful: *success,
                roll: *roll,
                minimum_roll: *target,
                re_rolled: *rerolled,
            }),
        GameEvent::GoForItRoll { player_id, target, roll, success, rerolled } =>
            Some(WireReport::GoForItRoll {
                player_id: player_id.clone(),
                successful: *success,
                roll: *roll,
                minimum_roll: *target,
                re_rolled: *rerolled,
            }),
        GameEvent::PickupRoll { player_id, target, roll, success, rerolled } =>
            Some(WireReport::PickupRoll {
                player_id: player_id.clone(),
                successful: *success,
                roll: *roll,
                minimum_roll: *target,
                re_rolled: *rerolled,
            }),
        GameEvent::CatchRoll { player_id, target, roll, success, rerolled } =>
            Some(WireReport::CatchRoll {
                player_id: player_id.clone(),
                successful: *success,
                roll: *roll,
                minimum_roll: *target,
                re_rolled: *rerolled,
            }),
        GameEvent::PassRoll { player_id, target, distance, roll, result, rerolled } =>
            Some(WireReport::PassRoll {
                player_id: player_id.clone(),
                successful: !matches!(result, ffb_model::enums::PassResult::Fumble),
                roll: *roll,
                minimum_roll: *target,
                re_rolled: *rerolled,
                passing_distance: Some(format!("{:?}", distance)),
                pass_result: format!("{:?}", result),
                hail_mary_pass: false,
                bomb: false,
            }),
        GameEvent::Injury { player_id, armor_roll, injury_roll, serious_injury, was_ko, was_cas } =>
            Some(WireReport::Injury {
                attacker_id: None,
                defender_id: player_id.clone(),
                armor_roll: armor_roll.map(|r| r.to_vec()).unwrap_or_default(),
                injury_roll: injury_roll.map(|r| r.to_vec()).unwrap_or_default(),
                armor_broken: *was_ko || *was_cas,
                injury: None,
                casualty_roll: vec![],
                serious_injury: serious_injury.as_ref().map(|s| format!("{:?}", s)),
            }),
        GameEvent::ReRoll { team_id, source, rerolled_action } =>
            Some(WireReport::ReRoll {
                team_id: team_id.clone(),
                source: source.name.clone(),
                re_rolled_action: Some(rerolled_action.clone()),
            }),
        GameEvent::SkillUse { player_id, skill_id, used } =>
            Some(WireReport::SkillUse {
                player_id: Some(player_id.clone()),
                skill: skill_id.to_string(),
                used: *used,
                skill_use: if *used { "USED".to_string() } else { "DECLINED".to_string() },
            }),
        GameEvent::PlayerAction { player_id, action } =>
            Some(WireReport::PlayerAction {
                player_id: player_id.clone(),
                player_action: format!("{:?}", action),
            }),
        GameEvent::TurnEnd { team_id, turn_nr } =>
            Some(WireReport::TurnEnd { team_id: team_id.clone(), turn_nr: *turn_nr }),
        GameEvent::KickoffResultEvent { result } =>
            Some(WireReport::KickoffResult { result: format!("{:?}", result) }),
        GameEvent::CoinThrow { home_won } =>
            Some(WireReport::CoinThrow { home_won: *home_won }),
        GameEvent::ReceiveChoice { team_id, receive } =>
            Some(WireReport::ReceiveChoice { team_id: team_id.clone(), receive: *receive }),
        GameEvent::Touchdown { player_id, .. } =>
            Some(WireReport::Touchdown { player_id: player_id.clone() }),
        GameEvent::Foul { defender_id, .. } =>
            Some(WireReport::Foul { defender_id: defender_id.clone() }),
        GameEvent::WeatherChange { weather } =>
            Some(WireReport::WeatherChange { weather: format!("{:?}", weather) }),
        event => {
            log::trace!("no wire report for event: {:?}", event);
            None
        }
    }
}

/// Convert a slice of `GameEvent`s to a `Vec<WireReport>`.
pub fn events_to_reports(events: &[GameEvent]) -> Vec<WireReport> {
    events.iter().filter_map(event_to_report).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{PassingDistance, PassResult, KickoffResult};
    use ffb_model::enums::ReRollSource;

    fn jsn(report: &WireReport) -> String { serde_json::to_string(report).unwrap() }
    fn sync_jsn(reports: Vec<WireReport>) -> String {
        serde_json::to_string(&OutgoingModelSync::new(1, reports)).unwrap()
    }

    // ── Outgoing wire format ───────────────────────────────────────────────────

    #[test]
    fn model_sync_has_all_required_keys() {
        let json = sync_jsn(vec![]);
        assert!(json.contains("\"netCommandId\":\"serverModelSync\""));
        assert!(json.contains("\"commandNr\":1"));
        assert!(json.contains("\"modelChangeList\""));
        assert!(json.contains("\"modelChangeArray\""));
        assert!(json.contains("\"reportList\""));
        assert!(json.contains("\"reports\""));
        assert!(json.contains("\"animation\":null"));
        assert!(json.contains("\"gameTime\":0"));
        assert!(json.contains("\"turnTime\":0"));
    }

    #[test]
    fn command_nr_is_embedded() {
        let json = serde_json::to_string(&OutgoingModelSync::new(42, vec![])).unwrap();
        assert!(json.contains("\"commandNr\":42"));
    }

    #[test]
    fn report_list_wraps_in_reports_key() {
        let list = WireReportList { reports: vec![WireReport::Block { defender_id: "p1".into() }] };
        let json = serde_json::to_string(&list).unwrap();
        assert!(json.starts_with("{\"reports\":["));
    }

    #[test]
    fn model_change_list_uses_camel_key() {
        let list = WireModelChangeList { model_change_array: vec![] };
        let json = serde_json::to_string(&list).unwrap();
        assert!(json.contains("\"modelChangeArray\""));
    }

    // ── WireReport serialization (report_id tag) ──────────────────────────────

    #[test]
    fn block_report_id_and_fields() {
        let json = jsn(&WireReport::Block { defender_id: "def".into() });
        assert!(json.contains("\"reportId\":\"BLOCK\""));
        assert!(json.contains("\"defenderId\":\"def\""));
    }

    #[test]
    fn block_roll_report_id_and_fields() {
        let json = jsn(&WireReport::BlockRoll {
            attacker_id: "a".into(), defender_id: "d".into(), nr_of_dice: 2,
            dice: vec![1, 3], selected_die: 1, own_choice: true, re_rolled: false,
        });
        assert!(json.contains("\"reportId\":\"BLOCK_ROLL\""));
        assert!(json.contains("\"nrOfDice\":2"));
        assert!(json.contains("\"ownChoice\":true"));
        assert!(json.contains("\"reRolled\":false"));
    }

    #[test]
    fn dodge_roll_report() {
        let json = jsn(&WireReport::DodgeRoll {
            player_id: "p".into(), successful: true, roll: 4, minimum_roll: 3, re_rolled: false,
        });
        assert!(json.contains("\"reportId\":\"DODGE_ROLL\""));
        assert!(json.contains("\"successful\":true"));
        assert!(json.contains("\"minimumRoll\":3"));
    }

    #[test]
    fn go_for_it_roll_report() {
        let json = jsn(&WireReport::GoForItRoll {
            player_id: "p".into(), successful: false, roll: 1, minimum_roll: 2, re_rolled: true,
        });
        assert!(json.contains("\"reportId\":\"GO_FOR_IT_ROLL\""));
        assert!(json.contains("\"reRolled\":true"));
    }

    #[test]
    fn pickup_roll_report() {
        let json = jsn(&WireReport::PickupRoll {
            player_id: "p".into(), successful: true, roll: 5, minimum_roll: 3, re_rolled: false,
        });
        assert!(json.contains("\"reportId\":\"PICKUP_ROLL\""));
    }

    #[test]
    fn catch_roll_report() {
        let json = jsn(&WireReport::CatchRoll {
            player_id: "p".into(), successful: true, roll: 4, minimum_roll: 2, re_rolled: false,
        });
        assert!(json.contains("\"reportId\":\"CATCH_ROLL\""));
    }

    #[test]
    fn pass_roll_report() {
        let json = jsn(&WireReport::PassRoll {
            player_id: "p".into(), successful: true, roll: 3, minimum_roll: 3, re_rolled: false,
            passing_distance: Some("SHORT".into()), pass_result: "ACCURATE".into(),
            hail_mary_pass: false, bomb: false,
        });
        assert!(json.contains("\"reportId\":\"PASS_ROLL\""));
        assert!(json.contains("\"passingDistance\""));
        assert!(json.contains("\"passResult\""));
    }

    #[test]
    fn injury_report_fields() {
        let json = jsn(&WireReport::Injury {
            attacker_id: Some("a".into()), defender_id: "d".into(),
            armor_roll: vec![3, 4], injury_roll: vec![2, 3],
            armor_broken: true, injury: None,
            casualty_roll: vec![], serious_injury: None,
        });
        assert!(json.contains("\"reportId\":\"INJURY\""));
        assert!(json.contains("\"armorRoll\""));
        assert!(json.contains("\"injuryRoll\""));
        assert!(json.contains("\"armorBroken\":true"));
    }

    #[test]
    fn re_roll_report() {
        let json = jsn(&WireReport::ReRoll {
            team_id: "team1".into(), source: "TEAM_RE_ROLL".into(),
            re_rolled_action: Some("DODGE".into()),
        });
        assert!(json.contains("\"reportId\":\"RE_ROLL\""));
        assert!(json.contains("\"teamId\":\"team1\""));
        assert!(json.contains("\"reRolledAction\""));
    }

    #[test]
    fn skill_use_report() {
        let json = jsn(&WireReport::SkillUse {
            player_id: Some("p".into()), skill: "52".into(), used: true, skill_use: "USED".into(),
        });
        assert!(json.contains("\"reportId\":\"SKILL_USE\""));
        assert!(json.contains("\"skillUse\":\"USED\""));
    }

    #[test]
    fn player_action_report() {
        let json = jsn(&WireReport::PlayerAction {
            player_id: "p".into(), player_action: "Move".into(),
        });
        assert!(json.contains("\"reportId\":\"PLAYER_ACTION\""));
        assert!(json.contains("\"playerAction\""));
    }

    #[test]
    fn turn_end_report() {
        let json = jsn(&WireReport::TurnEnd { team_id: "t".into(), turn_nr: 3 });
        assert!(json.contains("\"reportId\":\"TURN_END\""));
        assert!(json.contains("\"turnNr\":3"));
    }

    #[test]
    fn kickoff_result_report() {
        let json = jsn(&WireReport::KickoffResult { result: "BLITZ".into() });
        assert!(json.contains("\"reportId\":\"KICKOFF_RESULT\""));
    }

    #[test]
    fn coin_throw_report_home_won() {
        let json = jsn(&WireReport::CoinThrow { home_won: true });
        assert!(json.contains("\"reportId\":\"COIN_THROW\""));
        assert!(json.contains("\"homeWon\":true"));
    }

    #[test]
    fn receive_choice_report() {
        let json = jsn(&WireReport::ReceiveChoice { team_id: "t".into(), receive: true });
        assert!(json.contains("\"reportId\":\"RECEIVE_CHOICE\""));
        assert!(json.contains("\"receive\":true"));
    }

    #[test]
    fn touchdown_report() {
        let json = jsn(&WireReport::Touchdown { player_id: "scorer".into() });
        assert!(json.contains("\"reportId\":\"TOUCHDOWN\""));
        assert!(json.contains("\"playerId\":\"scorer\""));
    }

    #[test]
    fn foul_report() {
        let json = jsn(&WireReport::Foul { defender_id: "victim".into() });
        assert!(json.contains("\"reportId\":\"FOUL\""));
        assert!(json.contains("\"defenderId\":\"victim\""));
    }

    #[test]
    fn weather_change_report() {
        let json = jsn(&WireReport::WeatherChange { weather: "NICE".into() });
        assert!(json.contains("\"reportId\":\"WEATHER_CHANGE\""));
    }

    // ── event_to_report conversion ────────────────────────────────────────────

    #[test]
    fn event_block_converts() {
        let event = GameEvent::Block { defender_id: "d".into() };
        let report = event_to_report(&event).unwrap();
        let json = jsn(&report);
        assert!(json.contains("\"BLOCK\""));
        assert!(json.contains("\"defenderId\":\"d\""));
    }

    #[test]
    fn event_block_roll_converts() {
        let event = GameEvent::BlockRoll {
            attacker_id: "a".into(), defender_id: "d".into(), nr_of_dice: 1,
            dice: vec![2], selected_index: 0, own_choice: false, rerolled: false,
        };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"BLOCK_ROLL\""));
        assert!(json.contains("\"attackerId\":\"a\""));
    }

    #[test]
    fn event_dodge_roll_converts() {
        let event = GameEvent::DodgeRoll { player_id: "p".into(), target: 3, roll: 4, success: true, rerolled: false };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"DODGE_ROLL\""));
        assert!(json.contains("\"successful\":true"));
    }

    #[test]
    fn event_go_for_it_converts() {
        let event = GameEvent::GoForItRoll { player_id: "p".into(), target: 2, roll: 1, success: false, rerolled: true };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"GO_FOR_IT_ROLL\""));
        assert!(json.contains("\"reRolled\":true"));
    }

    #[test]
    fn event_pickup_converts() {
        let event = GameEvent::PickupRoll { player_id: "p".into(), target: 3, roll: 5, success: true, rerolled: false };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"PICKUP_ROLL\""));
    }

    #[test]
    fn event_catch_converts() {
        let event = GameEvent::CatchRoll { player_id: "p".into(), target: 2, roll: 4, success: true, rerolled: false };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"CATCH_ROLL\""));
    }

    #[test]
    fn event_pass_roll_converts() {
        let event = GameEvent::PassRoll {
            player_id: "p".into(), target: 3, distance: PassingDistance::ShortPass,
            roll: 4, result: PassResult::Complete, rerolled: false,
        };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"PASS_ROLL\""));
        assert!(json.contains("\"successful\":true"));
    }

    #[test]
    fn event_pass_fumble_is_not_successful() {
        let event = GameEvent::PassRoll {
            player_id: "p".into(), target: 3, distance: PassingDistance::ShortPass,
            roll: 1, result: PassResult::Fumble, rerolled: false,
        };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"successful\":false"));
    }

    #[test]
    fn event_injury_converts_with_armor_roll() {
        let event = GameEvent::Injury {
            player_id: "hurt".into(),
            armor_roll: Some([3, 4]),
            injury_roll: Some([1, 2]),
            serious_injury: None,
            was_ko: true,
            was_cas: false,
        };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"INJURY\""));
        assert!(json.contains("\"armorBroken\":true"));
        assert!(json.contains("[3,4]"));
    }

    #[test]
    fn event_injury_converts_without_armor_roll() {
        let event = GameEvent::Injury {
            player_id: "p".into(),
            armor_roll: None,
            injury_roll: None,
            serious_injury: None,
            was_ko: false,
            was_cas: false,
        };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"INJURY\""));
        assert!(json.contains("\"armorRoll\":[]"));
    }

    #[test]
    fn event_reroll_converts() {
        let event = GameEvent::ReRoll {
            team_id: "home".into(),
            source: ReRollSource::new("teamReRoll"),
            rerolled_action: "DODGE".into(),
        };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"RE_ROLL\""));
        assert!(json.contains("\"teamId\":\"home\""));
        assert!(json.contains("teamReRoll"));
    }

    #[test]
    fn event_skill_use_converts() {
        let event = GameEvent::SkillUse { player_id: "p".into(), skill_id: 52, used: true };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"SKILL_USE\""));
        assert!(json.contains("\"used\":true"));
        assert!(json.contains("\"skillUse\":\"USED\""));
    }

    #[test]
    fn event_skill_declined_converts() {
        let event = GameEvent::SkillUse { player_id: "p".into(), skill_id: 1, used: false };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"skillUse\":\"DECLINED\""));
    }

    #[test]
    fn event_player_action_converts() {
        let event = GameEvent::PlayerAction {
            player_id: "p".into(),
            action: ffb_model::enums::PlayerAction::Move,
        };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"PLAYER_ACTION\""));
    }

    #[test]
    fn event_turn_end_converts() {
        let event = GameEvent::TurnEnd { team_id: "home".into(), turn_nr: 4 };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"TURN_END\""));
        assert!(json.contains("\"turnNr\":4"));
    }

    #[test]
    fn event_coin_throw_converts() {
        let event = GameEvent::CoinThrow { home_won: false };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"COIN_THROW\""));
        assert!(json.contains("\"homeWon\":false"));
    }

    #[test]
    fn event_receive_choice_converts() {
        let event = GameEvent::ReceiveChoice { team_id: "away".into(), receive: false };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"RECEIVE_CHOICE\""));
    }

    #[test]
    fn event_touchdown_converts() {
        let event = GameEvent::Touchdown {
            player_id: "scorer".into(),
            coord: ffb_model::types::FieldCoordinate { x: 0, y: 5 },
        };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"TOUCHDOWN\""));
        assert!(json.contains("\"playerId\":\"scorer\""));
    }

    #[test]
    fn event_foul_converts() {
        let event = GameEvent::Foul {
            attacker_id: "fouler".into(),
            defender_id: "prone".into(),
        };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"FOUL\""));
        assert!(json.contains("\"defenderId\":\"prone\""));
    }

    #[test]
    fn event_kickoff_result_converts() {
        let event = GameEvent::KickoffResultEvent { result: KickoffResult::Blitz };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"KICKOFF_RESULT\""));
    }

    #[test]
    fn unknown_event_returns_none() {
        let event = GameEvent::PlayerMoved {
            player_id: "p".into(),
            coord: ffb_model::types::FieldCoordinate { x: 3, y: 4 },
        };
        assert!(event_to_report(&event).is_none());
    }

    #[test]
    fn events_to_reports_filters_unknowns() {
        let events = vec![
            GameEvent::CoinThrow { home_won: true },
            GameEvent::PlayerMoved {
                player_id: "p".into(),
                coord: ffb_model::types::FieldCoordinate { x: 1, y: 1 },
            },
            GameEvent::Block { defender_id: "d".into() },
        ];
        let reports = events_to_reports(&events);
        assert_eq!(reports.len(), 2);
    }
}
