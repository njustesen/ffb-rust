use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;
use crate::report::report_skill_roll::ReportSkillRoll;

/// 1:1 translation of `ReportDauntlessRoll.java`.
/// Extends `ReportSkillRoll`; adds strength and optional defender id.
#[derive(Debug, Clone)]
pub struct ReportDauntlessRoll {
    pub base: ReportSkillRoll,
    /// Translated from `fStrength`.
    pub strength: i32,
    /// Translated from `defenderId`.
    pub defender_id: Option<String>,
}

impl ReportDauntlessRoll {
    pub fn new(
        player_id: Option<String>,
        successful: bool,
        roll: i32,
        minimum_roll: i32,
        re_rolled: bool,
        strength: i32,
        defender_id: Option<String>,
    ) -> Self {
        Self {
            base: ReportSkillRoll::new(player_id, successful, roll, minimum_roll, re_rolled, vec![]),
            strength,
            defender_id,
        }
    }

    pub fn get_strength(&self) -> i32 {
        self.strength
    }

    pub fn get_defender_id(&self) -> Option<&str> {
        self.defender_id.as_deref()
    }
}

impl IReport for ReportDauntlessRoll {
    fn get_id(&self) -> ReportId {
        ReportId::DAUNTLESS_ROLL
    }
}

impl ReportDauntlessRoll {
    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "playerId": self.base.player_id,
            "successful": self.base.successful,
            "roll": self.base.roll,
            "minimumRoll": self.base.minimum_roll,
            "reRolled": self.base.re_rolled,
            "rollModifiers": self.base.roll_modifier_names,
            "strength": self.strength,
            "defenderId": self.defender_id,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        use crate::report::report_skill_roll::ReportSkillRoll;
        Self {
            base: ReportSkillRoll {
                player_id: json["playerId"].as_str().map(str::to_string),
                successful: json["successful"].as_bool().unwrap_or(false),
                roll: json["roll"].as_i64().unwrap_or(0) as i32,
                minimum_roll: json["minimumRoll"].as_i64().unwrap_or(0) as i32,
                re_rolled: json["reRolled"].as_bool().unwrap_or(false),
                roll_modifier_names: json["rollModifiers"].as_array().map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_string)).collect()).unwrap_or_default(),
            },
            strength: json["strength"].as_i64().unwrap_or(0) as i32,
            defender_id: json["defenderId"].as_str().map(str::to_string),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportDauntlessRoll {
        ReportDauntlessRoll::new(Some("p1".into()), true, 5, 3, false, 4, Some("d1".into()))
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::DAUNTLESS_ROLL);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "dauntlessRoll");
    }

    #[test]
    fn field_getters() {
        let r = make();
        assert_eq!(r.get_strength(), 4);
        assert_eq!(r.get_defender_id(), Some("d1"));
        assert!(r.base.is_successful());
    }

    #[test]
    fn no_defender_id() {
        let r = ReportDauntlessRoll::new(Some("p2".into()), false, 2, 4, false, 3, None);
        assert_eq!(r.get_defender_id(), None);
        assert_eq!(r.get_strength(), 3);
    }

    #[test]
    fn rerolled_dauntless() {
        let r = ReportDauntlessRoll::new(Some("p1".into()), true, 6, 3, true, 5, Some("def2".into()));
        assert!(r.base.is_re_rolled());
        assert_eq!(r.get_defender_id(), Some("def2"));
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportDauntlessRoll::from_json(&json);
        assert_eq!(restored.base.player_id, original.base.player_id);
        assert_eq!(restored.base.successful, original.base.successful);
        assert_eq!(restored.base.roll, original.base.roll);
        assert_eq!(restored.base.minimum_roll, original.base.minimum_roll);
        assert_eq!(restored.base.re_rolled, original.base.re_rolled);
        assert_eq!(restored.base.roll_modifier_names, original.base.roll_modifier_names);
        assert_eq!(restored.strength, original.strength);
        assert_eq!(restored.defender_id, original.defender_id);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("dauntlessRoll"));
    }
}
