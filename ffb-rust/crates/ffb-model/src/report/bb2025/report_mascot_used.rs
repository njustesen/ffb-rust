use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportMascotUsed.java`.
#[derive(Debug, Clone)]
pub struct ReportMascotUsed {
    pub team_id: String,
    pub minimum_roll: i32,
    pub roll: i32,
    pub successful: bool,
    pub fallback: bool,
}

impl ReportMascotUsed {
    pub fn new(team_id: String, minimum_roll: i32, roll: i32, successful: bool, fallback: bool) -> Self {
        Self { team_id, minimum_roll, roll, successful, fallback }
    }

    pub fn get_team_id(&self) -> &str { &self.team_id }
    pub fn get_minimum_roll(&self) -> i32 { self.minimum_roll }
    pub fn get_roll(&self) -> i32 { self.roll }
    pub fn is_successful(&self) -> bool { self.successful }
    pub fn is_fallback(&self) -> bool { self.fallback }

    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "teamId": self.team_id,
            "minimumRoll": self.minimum_roll,
            "roll": self.roll,
            "successful": self.successful,
            "reRollUsed": self.fallback,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            team_id: json["teamId"].as_str().unwrap_or("").to_string(),
            minimum_roll: json["minimumRoll"].as_i64().unwrap_or(0) as i32,
            roll: json["roll"].as_i64().unwrap_or(0) as i32,
            successful: json["successful"].as_bool().unwrap_or(false),
            fallback: json["reRollUsed"].as_bool().unwrap_or(false),
        }
    }
}

impl IReport for ReportMascotUsed {
    fn get_id(&self) -> ReportId { ReportId::MASCOT_USED }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportMascotUsed {
        ReportMascotUsed::new("team1".into(), 4, 5, true, false)
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::MASCOT_USED);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "mascotUsed");
    }

    #[test]
    fn fields() {
        let r = make();
        assert_eq!(r.get_team_id(), "team1");
        assert!(r.is_successful());
        assert!(!r.is_fallback());
    }

    #[test]
    fn minimum_roll_and_roll() {
        let r = make();
        assert_eq!(r.get_minimum_roll(), 4);
        assert_eq!(r.get_roll(), 5);
    }

    #[test]
    fn fallback_and_unsuccessful() {
        let r = ReportMascotUsed::new("team2".into(), 5, 3, false, true);
        assert!(!r.is_successful());
        assert!(r.is_fallback());
        assert_eq!(r.get_team_id(), "team2");
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportMascotUsed::from_json(&json);
        assert_eq!(restored.team_id, original.team_id);
        assert_eq!(restored.minimum_roll, original.minimum_roll);
        assert_eq!(restored.roll, original.roll);
        assert_eq!(restored.successful, original.successful);
        assert_eq!(restored.fallback, original.fallback);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("mascotUsed"));
    }
}
