use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;
use crate::report::report_skill_roll::ReportSkillRoll;

/// 1:1 translation of `ReportBreatheFire.java` — extends ReportSkillRoll.
#[derive(Debug, Clone)]
pub struct ReportBreatheFire {
    pub base: ReportSkillRoll,
    pub defender_id: Option<String>,
    pub strong_opponent: bool,
    pub result: String,
}

impl ReportBreatheFire {
    pub fn new(
        player_id: Option<String>,
        successful: bool,
        roll: i32,
        minimum_roll: i32,
        re_rolled: bool,
        defender_id: Option<String>,
        strong_opponent: bool,
        result: String,
    ) -> Self {
        Self {
            base: ReportSkillRoll::new(player_id, successful, roll, minimum_roll, re_rolled, vec![]),
            defender_id,
            strong_opponent,
            result,
        }
    }

    pub fn get_player_id(&self) -> Option<&str> { self.base.get_player_id() }
    pub fn is_successful(&self) -> bool { self.base.is_successful() }
    pub fn get_roll(&self) -> i32 { self.base.get_roll() }
    pub fn get_minimum_roll(&self) -> i32 { self.base.get_minimum_roll() }
    pub fn is_re_rolled(&self) -> bool { self.base.is_re_rolled() }
    pub fn get_defender_id(&self) -> Option<&str> { self.defender_id.as_deref() }
    pub fn is_strong_opponent(&self) -> bool { self.strong_opponent }
    pub fn get_result(&self) -> &str { &self.result }
}

impl IReport for ReportBreatheFire {
    fn get_id(&self) -> ReportId { ReportId::BREATHE_FIRE }
}

impl ReportBreatheFire {
    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "playerId": self.base.player_id,
            "successful": self.base.successful,
            "roll": self.base.roll,
            "minimumRoll": self.base.minimum_roll,
            "reRolled": self.base.re_rolled,
            "rollModifiers": self.base.roll_modifier_names,
            "defenderId": self.defender_id,
            "strongOpponent": self.strong_opponent,
            "status": self.result,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            base: ReportSkillRoll::new(
                json["playerId"].as_str().map(str::to_string),
                json["successful"].as_bool().unwrap_or(false),
                json["roll"].as_i64().unwrap_or(0) as i32,
                json["minimumRoll"].as_i64().unwrap_or(0) as i32,
                json["reRolled"].as_bool().unwrap_or(false),
                json["rollModifiers"].as_array().map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_string)).collect()).unwrap_or_default(),
            ),
            defender_id: json["defenderId"].as_str().map(str::to_string),
            strong_opponent: json["strongOpponent"].as_bool().unwrap_or(false),
            result: json["status"].as_str().unwrap_or("").to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportBreatheFire {
        ReportBreatheFire::new(
            Some("p1".into()), true, 4, 2, false,
            Some("d1".into()), false, "HIT".into(),
        )
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::BREATHE_FIRE); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "breatheFire"); }

    #[test]
    fn get_result() { assert_eq!(make().get_result(), "HIT"); }

    #[test]
    fn get_defender_id_and_successful() {
        assert_eq!(make().get_defender_id(), Some("d1"));
        assert!(make().is_successful());
    }

    #[test]
    fn strong_opponent_false() { assert!(!make().is_strong_opponent()); }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportBreatheFire::from_json(&json);
        assert_eq!(restored.base.player_id, original.base.player_id);
        assert_eq!(restored.base.successful, original.base.successful);
        assert_eq!(restored.base.roll, original.base.roll);
        assert_eq!(restored.base.minimum_roll, original.base.minimum_roll);
        assert_eq!(restored.base.re_rolled, original.base.re_rolled);
        assert_eq!(restored.defender_id, original.defender_id);
        assert_eq!(restored.strong_opponent, original.strong_opponent);
        assert_eq!(restored.result, original.result);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("breatheFire"));
    }
}
