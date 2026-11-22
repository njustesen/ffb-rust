use serde::{Deserialize, Serialize};
use crate::enums::{ApothecaryType, LeaderState, PlayerAction};
use crate::model::inducement_set::InducementSet;

/// Per-team per-half turn state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnData {
    /// Java: `fHomeData` -- which team this turn data belongs to (written into the JSON).
    #[serde(default)]
    pub home_data: bool,
    pub turn_nr: i32,
    pub first_turn_after_kickoff: bool,
    pub turn_started: bool,

    pub rerolls: i32,
    pub single_use_rerolls: i32,
    pub rerolls_brilliant_coaching_one_drive: i32,
    pub reroll_show_star_one_drive: i32,
    pub rerolls_pump_up_the_crowd_one_drive: i32,

    pub apothecaries: i32,
    pub wandering_apothecaries: i32,
    pub plague_doctors: i32,

    pub blitz_used: bool,
    pub foul_used: bool,
    pub reroll_used: bool,
    pub hand_over_used: bool,
    pub pass_used: bool,
    pub ttm_used: bool,
    pub ktm_used: bool,
    pub bomb_used: bool,
    pub secure_the_ball_used: bool,
    pub punt_used: bool,
    pub coach_banned: bool,

    pub leader_state: LeaderState,
    pub lord_of_chaos_state: LeaderState,
    /// QuickSnap kickoff event bonus: +1 MA for all players this turn.
    pub quick_snap_bonus: i32,
    /// Players who have completed their action this turn (IDs). Cleared each turn.
    pub acted_player_ids: Vec<String>,
    /// Eligible player list cached at turn start (Java parity: computed once when
    /// `acted_player_ids` is empty, never updated mid-turn). Cleared each turn.
    pub turn_eligible_cache: Vec<(String, Vec<PlayerAction>)>,
    /// Java: `TurnData.getInducementSet()` — inducements, cards, and prayers for this team.
    pub inducement_set: InducementSet,
}

impl TurnData {
    pub fn new() -> Self {
        TurnData {
            home_data: false,
            turn_nr: 0,
            first_turn_after_kickoff: false,
            turn_started: false,
            rerolls: 0,
            single_use_rerolls: 0,
            rerolls_brilliant_coaching_one_drive: 0,
            reroll_show_star_one_drive: 0,
            rerolls_pump_up_the_crowd_one_drive: 0,
            apothecaries: 0,
            wandering_apothecaries: 0,
            plague_doctors: 0,
            blitz_used: false,
            foul_used: false,
            reroll_used: false,
            hand_over_used: false,
            pass_used: false,
            ttm_used: false,
            ktm_used: false,
            bomb_used: false,
            secure_the_ball_used: false,
            punt_used: false,
            coach_banned: false,
            leader_state: LeaderState::None,
            lord_of_chaos_state: LeaderState::None,
            quick_snap_bonus: 0,
            acted_player_ids: Vec::new(),
            turn_eligible_cache: Vec::new(),
            inducement_set: InducementSet::new(),
        }
    }

    /// Java: TurnData.useApothecary(ApothecaryType) — consume one apothecary slot.
    pub fn use_apothecary(&mut self, apothecary_type: ApothecaryType) {
        if apothecary_type == ApothecaryType::Plague {
            if self.plague_doctors > 0 {
                self.plague_doctors -= 1;
            }
            return;
        }
        if (self.apothecaries <= self.wandering_apothecaries || apothecary_type == ApothecaryType::Wandering)
            && self.wandering_apothecaries > 0
        {
            self.wandering_apothecaries -= 1;
        }
        if self.apothecaries > 0 {
            self.apothecaries -= 1;
        }
    }

    pub fn reset_for_turn(&mut self) {
        self.blitz_used = false;
        self.foul_used = false;
        self.reroll_used = false;
        self.hand_over_used = false;
        self.pass_used = false;
        self.ttm_used = false;
        self.ktm_used = false;
        self.bomb_used = false;
        self.secure_the_ball_used = false;
        self.punt_used = false;
        // coach_banned persists for the entire drive — NOT reset per turn (matches Java TurnData.startTurn())
        self.quick_snap_bonus = 0;
        self.acted_player_ids.clear();
        self.turn_eligible_cache.clear();
    }
}

impl TurnData {
    /// Java: `TurnData.toJsonValue()` (BACKLOG §O). Java writes neither `singleUseReRolls` nor
    /// `reRollShowStarOneDrive`; the leader state as `LeaderState.getName()`.
    pub fn to_json_value(&self, rules: crate::enums::Rules) -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({
            "homeData": self.home_data,
            "turnStarted": self.turn_started,
            "turnNr": self.turn_nr,
            "firstTurnAfterKickoff": self.first_turn_after_kickoff,
            "reRolls": self.rerolls,
            "rerollBrilliantCoachingOneDrive": self.rerolls_brilliant_coaching_one_drive,
            "apothecaries": self.apothecaries,
            "blitzUsed": self.blitz_used,
            "foulUsed": self.foul_used,
            "reRollUsed": self.reroll_used,
            "handOverUsed": self.hand_over_used,
            "passUsed": self.pass_used,
            "coachBanned": self.coach_banned,
            "ktmUsed": self.ktm_used,
            "bombUsed": self.bomb_used,
            "secureTheBallUsed": self.secure_the_ball_used,
            "leaderState": self.leader_state.name(),
            "inducementSet": self.inducement_set.to_json_value(rules)?,
            "wanderingApothecaries": self.wandering_apothecaries,
            "rerollPumpUpTheCrowdOneDrive": self.rerolls_pump_up_the_crowd_one_drive,
            "plagueDoctors": self.plague_doctors,
            "ttmUsed": self.ttm_used,
            "puntUsed": self.punt_used,
        }))
    }

    /// Java: `TurnData.initFrom(IFactorySource, JsonValue)` in a game of `rules` (BACKLOG §O).
    /// Primitive booleans unboxed from `JsonBooleanOption` (`Err` when absent, as Java's NPE);
    /// `ktmUsed`/`bombUsed`/`coachBanned`/`secureTheBallUsed`/`ttmUsed`/`puntUsed` are null-safe;
    /// ints absent -> 0; a NEW `InducementSet` initialised from `inducementSet`. Fields Java does
    /// not read keep their value. `Err` for a null leader state (`LeaderStateFactory.forName`
    /// null / absent), which the Rust field cannot hold.
    pub fn init_from(
        &mut self,
        rules: crate::enums::Rules,
        options: &crate::model::game_options::GameOptions,
        json: &serde_json::Value,
    ) -> Result<(), String> {
        use crate::util::java_json::{
            get_boolean, get_boolean_object, get_enum_name, get_int, java_equals_ignore_case, to_json_object,
        };
        let obj = to_json_object(json)?;
        self.home_data = get_boolean(obj, "homeData")?;
        self.turn_started = get_boolean(obj, "turnStarted")?;
        self.turn_nr = get_int(obj, "turnNr")?;
        self.first_turn_after_kickoff = get_boolean(obj, "firstTurnAfterKickoff")?;
        self.rerolls = get_int(obj, "reRolls")?;
        self.rerolls_brilliant_coaching_one_drive = get_int(obj, "rerollBrilliantCoachingOneDrive")?;
        self.rerolls_pump_up_the_crowd_one_drive = get_int(obj, "rerollPumpUpTheCrowdOneDrive")?;
        self.apothecaries = get_int(obj, "apothecaries")?;
        self.blitz_used = get_boolean(obj, "blitzUsed")?;
        self.foul_used = get_boolean(obj, "foulUsed")?;
        self.reroll_used = get_boolean(obj, "reRollUsed")?;
        self.hand_over_used = get_boolean(obj, "handOverUsed")?;
        self.pass_used = get_boolean(obj, "passUsed")?;

        self.ktm_used = get_boolean_object(obj, "ktmUsed")? == Some(true);
        self.bomb_used = get_boolean_object(obj, "bombUsed")? == Some(true);
        self.coach_banned = get_boolean_object(obj, "coachBanned")? == Some(true);
        self.secure_the_ball_used = get_boolean_object(obj, "secureTheBallUsed")? == Some(true);
        self.ttm_used = get_boolean_object(obj, "ttmUsed")? == Some(true);
        self.punt_used = get_boolean_object(obj, "puntUsed")? == Some(true);

        let leader_name = get_enum_name(obj, "leaderState")?;
        self.leader_state = [LeaderState::None, LeaderState::Available, LeaderState::Used]
            .into_iter()
            .find(|s| leader_name.is_some_and(|n| java_equals_ignore_case(s.name(), n)))
            .ok_or_else(|| format!("TurnData.initFrom: leaderState {leader_name:?} is null"))?;
        let mut inducement_set = InducementSet::new();
        // IJsonOption.INDUCEMENT_SET.getFrom -> UtilJson.toJsonObject(null) throws when absent
        inducement_set.init_from(rules, options, obj.get("inducementSet").unwrap_or(&serde_json::Value::Null))?;
        self.inducement_set = inducement_set;

        self.wandering_apothecaries = get_int(obj, "wanderingApothecaries")?;
        self.plague_doctors = get_int(obj, "plagueDoctors")?;
        Ok(())
    }
}

impl Default for TurnData {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn java_json_round_trip() {
        use crate::enums::Rules;
        use crate::model::game_options::GameOptions;
        let input = serde_json::json!({
            "homeData": true, "turnStarted": false, "turnNr": 3, "firstTurnAfterKickoff": true,
            "reRolls": 2, "rerollBrilliantCoachingOneDrive": 1, "apothecaries": 1,
            "blitzUsed": true, "foulUsed": false, "reRollUsed": false, "handOverUsed": false,
            "passUsed": false, "leaderState": "AVAILABLE",
            "inducementSet": {
                "inducementArray": [{"inducementType": "bribes", "value": 2, "uses": 1}],
                "cardsAvailable": [], "cardsActive": [], "cardsDeactivated": [],
                "prayers": ["iron man"]
            },
            "wanderingApothecaries": 1, "plagueDoctors": 0, "ktmUsed": true,
            "initialAdjacentPartnerIds": ["ignored"]
        });
        let mut td = TurnData::new();
        td.init_from(Rules::Bb2025, &GameOptions::new(), &input).unwrap();
        assert_eq!(td.leader_state, LeaderState::Available);
        assert!(td.inducement_set.has_prayer("IRON_MAN"));
        let out = td.to_json_value(Rules::Bb2025).unwrap();
        // Java defaults for the null-safe booleans; absent ints -> 0; names canonical.
        assert_eq!(out["leaderState"], "available");
        assert_eq!(out["coachBanned"], false);
        assert_eq!(out["ktmUsed"], true);
        assert_eq!(out["rerollPumpUpTheCrowdOneDrive"], 0);
        assert_eq!(out["inducementSet"]["prayers"], serde_json::json!(["Iron Man"]));
        assert_eq!(out["inducementSet"]["inducementArray"][0]["inducementType"], "bribes");
        assert!(out["inducementSet"].get("starPlayerPositionIds").is_none());
        // a missing primitive boolean is a Java NPE
        let mut bad = input.clone();
        bad.as_object_mut().unwrap().remove("passUsed");
        assert!(TurnData::new().init_from(Rules::Bb2025, &GameOptions::new(), &bad).is_err());
    }

    #[test]
    fn reset_clears_flags() {
        let mut td = TurnData::new();
        td.blitz_used = true;
        td.pass_used = true;
        td.reset_for_turn();
        assert!(!td.blitz_used);
        assert!(!td.pass_used);
    }

    #[test]
    fn serde_round_trip() {
        let td = TurnData::new();
        let json = serde_json::to_string(&td).unwrap();
        let back: TurnData = serde_json::from_str(&json).unwrap();
        assert_eq!(td.turn_nr, back.turn_nr);
        assert_eq!(td.leader_state, back.leader_state);
    }

    #[test]
    fn reset_does_not_clear_rerolls() {
        let mut td = TurnData::new();
        td.rerolls = 3;
        td.apothecaries = 1;
        td.blitz_used = true;
        td.reset_for_turn();
        assert_eq!(td.rerolls, 3);
        assert_eq!(td.apothecaries, 1);
        assert!(!td.blitz_used);
    }

    #[test]
    fn use_apothecary_team_decrements_team() {
        let mut td = TurnData::new();
        td.apothecaries = 2;
        td.use_apothecary(ApothecaryType::Team);
        assert_eq!(td.apothecaries, 1);
        assert_eq!(td.wandering_apothecaries, 0);
    }

    #[test]
    fn use_apothecary_team_uses_wandering_when_wandering_ge_team() {
        let mut td = TurnData::new();
        td.apothecaries = 1;
        td.wandering_apothecaries = 2;
        td.use_apothecary(ApothecaryType::Team);
        // wandering >= team → decrement wandering, also decrement team
        assert_eq!(td.wandering_apothecaries, 1);
        assert_eq!(td.apothecaries, 0);
    }

    #[test]
    fn use_apothecary_wandering_always_decrements_wandering_and_team() {
        let mut td = TurnData::new();
        td.apothecaries = 1;
        td.wandering_apothecaries = 1;
        td.use_apothecary(ApothecaryType::Wandering);
        assert_eq!(td.wandering_apothecaries, 0);
        assert_eq!(td.apothecaries, 0);
    }

    #[test]
    fn use_apothecary_plague_decrements_plague_doctors_only() {
        let mut td = TurnData::new();
        td.apothecaries = 1;
        td.plague_doctors = 2;
        td.use_apothecary(ApothecaryType::Plague);
        assert_eq!(td.plague_doctors, 1);
        assert_eq!(td.apothecaries, 1);
    }

    #[test]
    fn use_apothecary_noop_when_none_available() {
        let mut td = TurnData::new();
        td.use_apothecary(ApothecaryType::Team);
        assert_eq!(td.apothecaries, 0);
        assert_eq!(td.wandering_apothecaries, 0);
    }

    #[test]
    fn quick_snap_bonus_is_reset_on_turn_end() {
        let mut td = TurnData::new();
        td.quick_snap_bonus = 1;
        td.reset_for_turn();
        assert_eq!(td.quick_snap_bonus, 0);
    }

    #[test]
    fn all_action_flags_reset_together() {
        let mut td = TurnData::new();
        td.blitz_used = true;
        td.foul_used = true;
        td.pass_used = true;
        td.hand_over_used = true;
        td.ttm_used = true;
        td.bomb_used = true;
        td.secure_the_ball_used = true;
        td.punt_used = true;
        td.reset_for_turn();
        assert!(!td.foul_used);
        assert!(!td.pass_used);
        assert!(!td.hand_over_used);
        assert!(!td.ttm_used);
        assert!(!td.bomb_used);
        assert!(!td.secure_the_ball_used);
        assert!(!td.punt_used);
    }
}
