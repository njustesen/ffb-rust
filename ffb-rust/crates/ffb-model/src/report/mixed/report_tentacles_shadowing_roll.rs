use crate::enums::SkillId;
use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportTentaclesShadowingRoll.java`.
#[derive(Debug, Clone)]
pub struct ReportTentaclesShadowingRoll {
    /// `fSkill`
    pub skill: Option<SkillId>,
    /// `fDefenderId`
    pub defender_id: Option<String>,
    /// `fRoll`
    pub roll: i32,
    /// `fSuccessful`
    pub successful: bool,
    /// `fMinimumRoll`
    pub minimum_roll: i32,
    /// `fReRolled`
    pub re_rolled: bool,
}

impl ReportTentaclesShadowingRoll {
    pub fn new(
        skill: Option<SkillId>,
        defender_id: Option<String>,
        roll: i32,
        successful: bool,
        minimum_roll: i32,
        re_rolled: bool,
    ) -> Self {
        Self { skill, defender_id, roll, successful, minimum_roll, re_rolled }
    }

    pub fn get_skill(&self) -> Option<SkillId> { self.skill }
    pub fn get_defender_id(&self) -> Option<&str> { self.defender_id.as_deref() }
    pub fn get_roll(&self) -> i32 { self.roll }
    pub fn is_successful(&self) -> bool { self.successful }
    pub fn get_minimum_roll(&self) -> i32 { self.minimum_roll }
    pub fn is_re_rolled(&self) -> bool { self.re_rolled }

    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "skill": self.skill.map(|s| s.class_name()),
            "defenderId": self.defender_id,
            "roll": self.roll,
            "successful": self.successful,
            "minimumRoll": self.minimum_roll,
            "reRolled": self.re_rolled,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            skill: json["skill"].as_str().and_then(SkillId::from_class_name),
            defender_id: json["defenderId"].as_str().map(str::to_string),
            roll: json["roll"].as_i64().unwrap_or(0) as i32,
            successful: json["successful"].as_bool().unwrap_or(false),
            minimum_roll: json["minimumRoll"].as_i64().unwrap_or(0) as i32,
            re_rolled: json["reRolled"].as_bool().unwrap_or(false),
        }
    }
}

impl IReport for ReportTentaclesShadowingRoll {
    fn get_id(&self) -> ReportId { ReportId::TENTACLES_SHADOWING_ROLL }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportTentaclesShadowingRoll {
        ReportTentaclesShadowingRoll::new(None, Some("d1".into()), 4, true, 3, false)
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::TENTACLES_SHADOWING_ROLL); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "tentaclesShadowingRoll"); }

    #[test]
    fn get_defender_id() { assert_eq!(make().get_defender_id(), Some("d1")); }

    #[test]
    fn is_successful() { assert!(make().is_successful()); }
    #[test]
    fn get_name_is_nonempty() {
        assert!(!make().get_name().is_empty());
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportTentaclesShadowingRoll::from_json(&json);
        assert_eq!(restored.skill, original.skill);
        assert_eq!(restored.defender_id, original.defender_id);
        assert_eq!(restored.roll, original.roll);
        assert_eq!(restored.successful, original.successful);
        assert_eq!(restored.minimum_roll, original.minimum_roll);
        assert_eq!(restored.re_rolled, original.re_rolled);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("tentaclesShadowingRoll"));
    }
}
