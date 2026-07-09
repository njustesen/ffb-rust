use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;
use crate::report::report_skill_roll::ReportSkillRoll;

/// 1:1 translation of `ReportConfusionRoll.java`.
/// Extends `ReportSkillRoll`; adds the confusion skill name.
#[derive(Debug, Clone)]
pub struct ReportConfusionRoll {
    pub base: ReportSkillRoll,
    /// Translated from `fConfusionSkill` (Skill → SkillId name as String).
    pub confusion_skill: Option<String>,
}

impl ReportConfusionRoll {
    pub fn new(
        player_id: Option<String>,
        successful: bool,
        roll: i32,
        minimum_roll: i32,
        re_rolled: bool,
        confusion_skill: Option<String>,
    ) -> Self {
        Self {
            base: ReportSkillRoll::new(player_id, successful, roll, minimum_roll, re_rolled, vec![]),
            confusion_skill,
        }
    }

    pub fn get_confusion_skill(&self) -> Option<&str> {
        self.confusion_skill.as_deref()
    }
}

impl IReport for ReportConfusionRoll {
    fn get_id(&self) -> ReportId {
        ReportId::CONFUSION_ROLL
    }
}

impl ReportConfusionRoll {
    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "playerId": self.base.player_id,
            "successful": self.base.successful,
            "roll": self.base.roll,
            "minimumRoll": self.base.minimum_roll,
            "reRolled": self.base.re_rolled,
            "rollModifiers": self.base.roll_modifier_names,
            "confusionSkill": self.confusion_skill,
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
            confusion_skill: json["confusionSkill"].as_str().map(str::to_string),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportConfusionRoll {
        ReportConfusionRoll::new(Some("p1".into()), true, 4, 2, false, Some("Confusion".into()))
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::CONFUSION_ROLL);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "confusionRoll");
    }

    #[test]
    fn confusion_skill_getter() {
        let r = make();
        assert_eq!(r.get_confusion_skill(), Some("Confusion"));
        assert!(r.base.is_successful());
        assert_eq!(r.base.get_roll(), 4);
    }

    #[test]
    fn no_confusion_skill() {
        let r = ReportConfusionRoll::new(Some("p2".into()), false, 1, 3, false, None);
        assert_eq!(r.get_confusion_skill(), None);
        assert!(!r.base.is_successful());
    }

    #[test]
    fn rerolled_confusion() {
        let r = ReportConfusionRoll::new(Some("p1".into()), true, 5, 2, true, Some("Bone Head".into()));
        assert!(r.base.is_re_rolled());
        assert_eq!(r.get_confusion_skill(), Some("Bone Head"));
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportConfusionRoll::from_json(&json);
        assert_eq!(restored.base.player_id, original.base.player_id);
        assert_eq!(restored.base.successful, original.base.successful);
        assert_eq!(restored.base.roll, original.base.roll);
        assert_eq!(restored.base.minimum_roll, original.base.minimum_roll);
        assert_eq!(restored.base.re_rolled, original.base.re_rolled);
        assert_eq!(restored.base.roll_modifier_names, original.base.roll_modifier_names);
        assert_eq!(restored.confusion_skill, original.confusion_skill);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("confusionRoll"));
    }
}
