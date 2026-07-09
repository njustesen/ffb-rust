use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;
use crate::report::report_skill_roll::ReportSkillRoll;

/// 1:1 translation of `ReportProjectileVomit.java` — extends ReportSkillRoll.
#[derive(Debug, Clone)]
pub struct ReportProjectileVomit {
    pub base: ReportSkillRoll,
    pub defender_id: Option<String>,
}

impl ReportProjectileVomit {
    pub fn new(
        player_id: Option<String>,
        successful: bool,
        roll: i32,
        minimum_roll: i32,
        re_rolled: bool,
        defender_id: Option<String>,
    ) -> Self {
        Self {
            base: ReportSkillRoll::new(player_id, successful, roll, minimum_roll, re_rolled, vec![]),
            defender_id,
        }
    }

    pub fn get_player_id(&self) -> Option<&str> { self.base.get_player_id() }
    pub fn is_successful(&self) -> bool { self.base.is_successful() }
    pub fn get_roll(&self) -> i32 { self.base.get_roll() }
    pub fn get_minimum_roll(&self) -> i32 { self.base.get_minimum_roll() }
    pub fn is_re_rolled(&self) -> bool { self.base.is_re_rolled() }
    pub fn get_defender_id(&self) -> Option<&str> { self.defender_id.as_deref() }
}

impl IReport for ReportProjectileVomit {
    fn get_id(&self) -> ReportId { ReportId::PROJECTILE_VOMIT }
}

impl ReportProjectileVomit {
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
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportProjectileVomit {
        ReportProjectileVomit::new(Some("p1".into()), true, 4, 2, false, Some("d1".into()))
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::PROJECTILE_VOMIT); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "projectileVomit"); }

    #[test]
    fn get_defender_id() { assert_eq!(make().get_defender_id(), Some("d1")); }

    #[test]
    fn minimum_roll_and_rerolled() {
        let r = ReportProjectileVomit::new(Some("p1".into()), true, 4, 3, true, None);
        assert_eq!(r.get_minimum_roll(), 3);
        assert!(r.is_re_rolled());
    }

    #[test]
    fn unsuccessful_no_defender() {
        let r = ReportProjectileVomit::new(None, false, 2, 4, false, None);
        assert!(!r.is_successful());
        assert!(r.get_defender_id().is_none());
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportProjectileVomit::from_json(&json);
        assert_eq!(restored.base.player_id, original.base.player_id);
        assert_eq!(restored.base.successful, original.base.successful);
        assert_eq!(restored.base.roll, original.base.roll);
        assert_eq!(restored.base.minimum_roll, original.base.minimum_roll);
        assert_eq!(restored.base.re_rolled, original.base.re_rolled);
        assert_eq!(restored.defender_id, original.defender_id);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("projectileVomit"));
    }
}
