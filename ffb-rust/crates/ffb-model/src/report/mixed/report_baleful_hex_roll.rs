use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;
use crate::report::report_skill_roll::ReportSkillRoll;

/// 1:1 translation of `ReportBalefulHexRoll.java` — extends ReportSkillRoll.
#[derive(Debug, Clone)]
pub struct ReportBalefulHexRoll {
    pub base: ReportSkillRoll,
    pub target: Option<String>,
}

impl ReportBalefulHexRoll {
    pub fn new(
        player_id: Option<String>,
        successful: bool,
        roll: i32,
        minimum_roll: i32,
        re_rolled: bool,
        target: Option<String>,
    ) -> Self {
        Self {
            base: ReportSkillRoll::new(player_id, successful, roll, minimum_roll, re_rolled, vec![]),
            target,
        }
    }

    pub fn get_player_id(&self) -> Option<&str> { self.base.get_player_id() }
    pub fn is_successful(&self) -> bool { self.base.is_successful() }
    pub fn get_roll(&self) -> i32 { self.base.get_roll() }
    pub fn get_minimum_roll(&self) -> i32 { self.base.get_minimum_roll() }
    pub fn is_re_rolled(&self) -> bool { self.base.is_re_rolled() }
    pub fn get_target(&self) -> Option<&str> { self.target.as_deref() }
}

impl IReport for ReportBalefulHexRoll {
    fn get_id(&self) -> ReportId { ReportId::BALEFUL_HEX }
}

impl ReportBalefulHexRoll {
    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "playerId": self.base.player_id,
            "successful": self.base.successful,
            "roll": self.base.roll,
            "minimumRoll": self.base.minimum_roll,
            "reRolled": self.base.re_rolled,
            "rollModifiers": self.base.roll_modifier_names,
            "targetPlayerId": self.target,
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
            target: json["targetPlayerId"].as_str().map(str::to_string),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportBalefulHexRoll {
        ReportBalefulHexRoll::new(Some("p1".into()), true, 4, 2, false, Some("t1".into()))
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::BALEFUL_HEX); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "balefulHex"); }

    #[test]
    fn get_target() { assert_eq!(make().get_target(), Some("t1")); }

    #[test]
    fn minimum_roll_and_rerolled() {
        let r = ReportBalefulHexRoll::new(Some("p1".into()), true, 4, 3, true, None);
        assert_eq!(r.get_minimum_roll(), 3);
        assert!(r.is_re_rolled());
    }

    #[test]
    fn unsuccessful_with_no_target() {
        let r = ReportBalefulHexRoll::new(None, false, 2, 4, false, None);
        assert!(!r.is_successful());
        assert_eq!(r.get_target(), None);
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportBalefulHexRoll::from_json(&json);
        assert_eq!(restored.base.player_id, original.base.player_id);
        assert_eq!(restored.base.successful, original.base.successful);
        assert_eq!(restored.base.roll, original.base.roll);
        assert_eq!(restored.base.minimum_roll, original.base.minimum_roll);
        assert_eq!(restored.base.re_rolled, original.base.re_rolled);
        assert_eq!(restored.target, original.target);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("balefulHex"));
    }
}
