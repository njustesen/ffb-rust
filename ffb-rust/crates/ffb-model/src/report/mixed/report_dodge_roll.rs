use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;
use crate::report::report_skill_roll::ReportSkillRoll;

/// 1:1 translation of `ReportDodgeRoll.java` — extends ReportSkillRoll.
#[derive(Debug, Clone)]
pub struct ReportDodgeRoll {
    pub base: ReportSkillRoll,
    pub stat_based_roll_modifier: Option<String>,
}

impl ReportDodgeRoll {
    pub fn new(
        player_id: Option<String>,
        successful: bool,
        roll: i32,
        minimum_roll: i32,
        re_rolled: bool,
        roll_modifiers: Vec<String>,
        stat_based_roll_modifier: Option<String>,
    ) -> Self {
        Self {
            base: ReportSkillRoll::new(player_id, successful, roll, minimum_roll, re_rolled, roll_modifiers),
            stat_based_roll_modifier,
        }
    }

    pub fn get_player_id(&self) -> Option<&str> { self.base.get_player_id() }
    pub fn is_successful(&self) -> bool { self.base.is_successful() }
    pub fn get_roll(&self) -> i32 { self.base.get_roll() }
    pub fn get_minimum_roll(&self) -> i32 { self.base.get_minimum_roll() }
    pub fn is_re_rolled(&self) -> bool { self.base.is_re_rolled() }
    pub fn get_roll_modifiers(&self) -> &[String] { self.base.get_roll_modifiers() }
    pub fn get_stat_based_roll_modifier(&self) -> Option<&str> { self.stat_based_roll_modifier.as_deref() }
}

impl IReport for ReportDodgeRoll {
    fn get_id(&self) -> ReportId { ReportId::DODGE_ROLL }
}

impl ReportDodgeRoll {
    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "playerId": self.base.player_id,
            "successful": self.base.successful,
            "roll": self.base.roll,
            "minimumRoll": self.base.minimum_roll,
            "reRolled": self.base.re_rolled,
            "rollModifiers": self.base.roll_modifier_names,
            "statBasedRollModifier": self.stat_based_roll_modifier,
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
            stat_based_roll_modifier: json["statBasedRollModifier"].as_str().map(str::to_string),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportDodgeRoll {
        ReportDodgeRoll::new(Some("p1".into()), true, 4, 2, false, vec![], Some("mod1".into()))
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::DODGE_ROLL); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "dodgeRoll"); }

    #[test]
    fn get_stat_based_roll_modifier() { assert_eq!(make().get_stat_based_roll_modifier(), Some("mod1")); }

    #[test]
    fn minimum_roll_and_rerolled() {
        let r = ReportDodgeRoll::new(Some("p1".into()), true, 4, 3, true, vec![], None);
        assert_eq!(r.get_minimum_roll(), 3);
        assert!(r.is_re_rolled());
    }

    #[test]
    fn unsuccessful_with_modifiers() {
        let r = ReportDodgeRoll::new(None, false, 2, 4, false, vec!["TackleZone".into()], None);
        assert!(!r.is_successful());
        assert_eq!(r.get_roll_modifiers().len(), 1);
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportDodgeRoll::from_json(&json);
        assert_eq!(restored.base.player_id, original.base.player_id);
        assert_eq!(restored.base.successful, original.base.successful);
        assert_eq!(restored.base.roll, original.base.roll);
        assert_eq!(restored.base.minimum_roll, original.base.minimum_roll);
        assert_eq!(restored.base.re_rolled, original.base.re_rolled);
        assert_eq!(restored.base.roll_modifier_names, original.base.roll_modifier_names);
        assert_eq!(restored.stat_based_roll_modifier, original.stat_based_roll_modifier);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("dodgeRoll"));
    }
}
