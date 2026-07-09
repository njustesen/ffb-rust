use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;
use crate::report::report_skill_roll::ReportSkillRoll;

/// 1:1 translation of `ReportRightStuffRoll.java` — extends ReportSkillRoll.
#[derive(Debug, Clone)]
pub struct ReportRightStuffRoll {
    pub base: ReportSkillRoll,
}

impl ReportRightStuffRoll {
    pub fn new(
        player_id: Option<String>,
        successful: bool,
        roll: i32,
        minimum_roll: i32,
        re_rolled: bool,
        roll_modifiers: Vec<String>,
    ) -> Self {
        Self {
            base: ReportSkillRoll::new(player_id, successful, roll, minimum_roll, re_rolled, roll_modifiers),
        }
    }

    pub fn get_player_id(&self) -> Option<&str> { self.base.get_player_id() }
    pub fn is_successful(&self) -> bool { self.base.is_successful() }
    pub fn get_roll(&self) -> i32 { self.base.get_roll() }
    pub fn get_minimum_roll(&self) -> i32 { self.base.get_minimum_roll() }
    pub fn is_re_rolled(&self) -> bool { self.base.is_re_rolled() }
    pub fn get_roll_modifiers(&self) -> &[String] { self.base.get_roll_modifiers() }

    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "playerId": self.base.player_id,
            "successful": self.base.successful,
            "roll": self.base.roll,
            "minimumRoll": self.base.minimum_roll,
            "reRolled": self.base.re_rolled,
            "rollModifiers": self.base.roll_modifier_names,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            base: crate::report::report_skill_roll::ReportSkillRoll {
                player_id: json["playerId"].as_str().map(str::to_string),
                successful: json["successful"].as_bool().unwrap_or(false),
                roll: json["roll"].as_i64().unwrap_or(0) as i32,
                minimum_roll: json["minimumRoll"].as_i64().unwrap_or(0) as i32,
                re_rolled: json["reRolled"].as_bool().unwrap_or(false),
                roll_modifier_names: json["rollModifiers"].as_array().map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_string)).collect()).unwrap_or_default(),
            },
        }
    }
}

impl IReport for ReportRightStuffRoll {
    fn get_id(&self) -> ReportId { ReportId::RIGHT_STUFF_ROLL }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportRightStuffRoll {
        ReportRightStuffRoll::new(Some("p1".into()), false, 2, 4, false, vec![])
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::RIGHT_STUFF_ROLL);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "rightStuffRoll");
    }

    #[test]
    fn is_successful() {
        assert!(!make().is_successful());
    }

    #[test]
    fn minimum_roll_and_rerolled() {
        let r = ReportRightStuffRoll::new(Some("p1".into()), true, 4, 3, true, vec![]);
        assert_eq!(r.get_minimum_roll(), 3);
        assert!(r.is_re_rolled());
    }

    #[test]
    fn unsuccessful_with_modifiers() {
        let r = ReportRightStuffRoll::new(None, false, 2, 4, false, vec!["TackleZone".into()]);
        assert!(!r.is_successful());
        assert_eq!(r.get_roll_modifiers().len(), 1);
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportRightStuffRoll::from_json(&json);
        assert_eq!(restored.base.player_id, original.base.player_id);
        assert_eq!(restored.base.successful, original.base.successful);
        assert_eq!(restored.base.roll, original.base.roll);
        assert_eq!(restored.base.minimum_roll, original.base.minimum_roll);
        assert_eq!(restored.base.re_rolled, original.base.re_rolled);
        assert_eq!(restored.base.roll_modifier_names, original.base.roll_modifier_names);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("rightStuffRoll"));
    }
}
