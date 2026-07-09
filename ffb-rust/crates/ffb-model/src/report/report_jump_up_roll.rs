use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;
use crate::report::report_skill_roll::ReportSkillRoll;

/// 1:1 translation of `ReportJumpUpRoll.java`.
/// Extends `ReportSkillRoll`; no additional fields.
#[derive(Debug, Clone)]
pub struct ReportJumpUpRoll {
    pub base: ReportSkillRoll,
}

impl ReportJumpUpRoll {
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
}

impl IReport for ReportJumpUpRoll {
    fn get_id(&self) -> ReportId {
        ReportId::JUMP_UP_ROLL
    }
}

impl ReportJumpUpRoll {
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
        use crate::report::report_skill_roll::ReportSkillRoll;
        Self {
            base: ReportSkillRoll {
                player_id: json["playerId"].as_str().map(str::to_string),
                successful: json["successful"].as_bool().unwrap_or(false),
                roll: json["roll"].as_i64().unwrap_or(0) as i32,
                minimum_roll: json["minimumRoll"].as_i64().unwrap_or(0) as i32,
                re_rolled: json["reRolled"].as_bool().unwrap_or(false),
                roll_modifier_names: json["rollModifiers"].as_array()
                    .map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_string)).collect())
                    .unwrap_or_default(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportJumpUpRoll {
        ReportJumpUpRoll::new(Some("p1".into()), false, 1, 3, true, vec![])
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::JUMP_UP_ROLL);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "jumpUpRoll");
    }

    #[test]
    fn base_fields() {
        let r = make();
        assert!(!r.base.is_successful());
        assert!(r.base.is_re_rolled());
        assert_eq!(r.base.get_roll(), 1);
    }

    #[test]
    fn minimum_roll_and_rerolled() {
        let r = ReportJumpUpRoll::new(Some("p1".into()), true, 4, 3, true, vec![]);
        assert_eq!(r.base.get_minimum_roll(), 3);
        assert!(r.base.is_re_rolled());
    }

    #[test]
    fn unsuccessful_with_modifiers() {
        let r = ReportJumpUpRoll::new(None, false, 2, 4, false, vec!["TackleZone".into()]);
        assert!(!r.base.is_successful());
        assert_eq!(r.base.get_roll_modifiers().len(), 1);
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportJumpUpRoll::from_json(&json);
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
        assert_eq!(json["reportId"].as_str(), Some("jumpUpRoll"));
    }
}
