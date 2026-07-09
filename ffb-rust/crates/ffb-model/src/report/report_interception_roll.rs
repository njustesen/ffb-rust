use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;
use crate::report::report_skill_roll::ReportSkillRoll;

/// 1:1 translation of `ReportInterceptionRoll.java`.
/// Extends `ReportSkillRoll`; adds bomb flag and ignore-agility flag.
#[derive(Debug, Clone)]
pub struct ReportInterceptionRoll {
    pub base: ReportSkillRoll,
    /// Translated from `fBomb`.
    pub bomb: bool,
    /// Translated from `ignoreAgility`.
    pub ignore_agility: bool,
}

impl ReportInterceptionRoll {
    pub fn new(
        player_id: Option<String>,
        successful: bool,
        roll: i32,
        minimum_roll: i32,
        re_rolled: bool,
        roll_modifiers: Vec<String>,
        bomb: bool,
        ignore_agility: bool,
    ) -> Self {
        Self {
            base: ReportSkillRoll::new(player_id, successful, roll, minimum_roll, re_rolled, roll_modifiers),
            bomb,
            ignore_agility,
        }
    }

    pub fn is_bomb(&self) -> bool {
        self.bomb
    }

    pub fn is_ignore_agility(&self) -> bool {
        self.ignore_agility
    }
}

impl IReport for ReportInterceptionRoll {
    fn get_id(&self) -> ReportId {
        ReportId::INTERCEPTION_ROLL
    }
}

impl ReportInterceptionRoll {
    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "playerId": self.base.player_id,
            "successful": self.base.successful,
            "roll": self.base.roll,
            "minimumRoll": self.base.minimum_roll,
            "reRolled": self.base.re_rolled,
            "rollModifiers": self.base.roll_modifier_names,
            "bomb": self.bomb,
            "ignoreAgility": self.ignore_agility,
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
            bomb: json["bomb"].as_bool().unwrap_or(false),
            ignore_agility: json["ignoreAgility"].as_bool().unwrap_or(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportInterceptionRoll {
        ReportInterceptionRoll::new(
            Some("p1".into()),
            false,
            3,
            5,
            false,
            vec![],
            true,
            false,
        )
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::INTERCEPTION_ROLL);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "interceptionRoll");
    }

    #[test]
    fn field_getters() {
        let r = make();
        assert!(r.is_bomb());
        assert!(!r.is_ignore_agility());
        assert!(!r.base.is_successful());
    }

    #[test]
    fn minimum_roll_and_rerolled() {
        let r = ReportInterceptionRoll::new(Some("p1".into()), true, 4, 3, true, vec![], false, false);
        assert_eq!(r.base.get_minimum_roll(), 3);
        assert!(r.base.is_re_rolled());
    }

    #[test]
    fn unsuccessful_with_modifiers() {
        let r = ReportInterceptionRoll::new(None, false, 2, 4, false, vec!["TackleZone".into()], false, true);
        assert!(!r.base.is_successful());
        assert_eq!(r.base.get_roll_modifiers().len(), 1);
        assert!(r.is_ignore_agility());
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportInterceptionRoll::from_json(&json);
        assert_eq!(restored.base.player_id, original.base.player_id);
        assert_eq!(restored.base.successful, original.base.successful);
        assert_eq!(restored.base.roll, original.base.roll);
        assert_eq!(restored.base.minimum_roll, original.base.minimum_roll);
        assert_eq!(restored.base.re_rolled, original.base.re_rolled);
        assert_eq!(restored.base.roll_modifier_names, original.base.roll_modifier_names);
        assert_eq!(restored.bomb, original.bomb);
        assert_eq!(restored.ignore_agility, original.ignore_agility);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("interceptionRoll"));
    }
}
