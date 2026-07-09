use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportTentaclesShadowingRoll.java`.
#[derive(Debug, Clone)]
pub struct ReportTentaclesShadowingRoll {
    pub skill: String,
    pub defender_id: String,
    pub roll: Vec<i32>,
    pub successful: bool,
    pub minimum_roll: i32,
    pub re_rolled: bool,
}

impl ReportTentaclesShadowingRoll {
    pub fn new(
        skill: String,
        defender_id: String,
        roll: Vec<i32>,
        successful: bool,
        minimum_roll: i32,
        re_rolled: bool,
    ) -> Self {
        Self { skill, defender_id, roll, successful, minimum_roll, re_rolled }
    }

    pub fn get_skill(&self) -> &str { &self.skill }
    pub fn get_defender_id(&self) -> &str { &self.defender_id }
    pub fn get_roll(&self) -> &[i32] { &self.roll }
    pub fn is_successful(&self) -> bool { self.successful }
    pub fn get_minimum_roll(&self) -> i32 { self.minimum_roll }
    pub fn is_re_rolled(&self) -> bool { self.re_rolled }

    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "skill": self.skill,
            "defenderId": self.defender_id,
            "tentacleRoll": self.roll,
            "successful": self.successful,
            "minimumRoll": self.minimum_roll,
            "reRolled": self.re_rolled,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            skill: json["skill"].as_str().unwrap_or("").to_string(),
            defender_id: json["defenderId"].as_str().unwrap_or("").to_string(),
            roll: json["tentacleRoll"].as_array().map(|a| a.iter().map(|v| v.as_i64().unwrap_or(0) as i32).collect()).unwrap_or_default(),
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
        ReportTentaclesShadowingRoll::new("Tentacles".into(), "d1".into(), vec![3, 4], false, 5, false)
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::TENTACLES_SHADOWING_ROLL);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "tentaclesShadowingRoll");
    }

    #[test]
    fn fields() {
        let r = make();
        assert_eq!(r.get_skill(), "Tentacles");
        assert_eq!(r.get_defender_id(), "d1");
        assert!(!r.is_successful());
    }

    #[test]
    fn minimum_roll_and_roll_values() {
        let r = make();
        assert_eq!(r.get_minimum_roll(), 5);
        assert_eq!(r.get_roll(), &[3, 4]);
    }

    #[test]
    fn rerolled_and_successful() {
        let r = ReportTentaclesShadowingRoll::new("Shadowing".into(), "d2".into(), vec![6], true, 4, true);
        assert!(r.is_successful());
        assert!(r.is_re_rolled());
        assert_eq!(r.get_skill(), "Shadowing");
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
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("tentaclesShadowingRoll"));
    }
}
