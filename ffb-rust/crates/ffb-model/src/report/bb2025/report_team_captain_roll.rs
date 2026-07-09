use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportTeamCaptainRoll.java`.
#[derive(Debug, Clone)]
pub struct ReportTeamCaptainRoll {
    pub team_id: String,
    pub minimum_roll: i32,
    pub roll: i32,
    pub successful: bool,
}

impl ReportTeamCaptainRoll {
    pub fn new(team_id: String, minimum_roll: i32, roll: i32, successful: bool) -> Self {
        Self { team_id, minimum_roll, roll, successful }
    }

    pub fn get_team_id(&self) -> &str { &self.team_id }
    pub fn get_minimum_roll(&self) -> i32 { self.minimum_roll }
    pub fn get_roll(&self) -> i32 { self.roll }
    pub fn is_successful(&self) -> bool { self.successful }

    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "teamId": self.team_id,
            "minimumRoll": self.minimum_roll,
            "roll": self.roll,
            "successful": self.successful,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            team_id: json["teamId"].as_str().unwrap_or("").to_string(),
            minimum_roll: json["minimumRoll"].as_i64().unwrap_or(0) as i32,
            roll: json["roll"].as_i64().unwrap_or(0) as i32,
            successful: json["successful"].as_bool().unwrap_or(false),
        }
    }
}

impl IReport for ReportTeamCaptainRoll {
    fn get_id(&self) -> ReportId { ReportId::TEAM_CAPTAIN_ROLL }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportTeamCaptainRoll {
        ReportTeamCaptainRoll::new("team1".into(), 4, 5, true)
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::TEAM_CAPTAIN_ROLL);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "teamCaptainRoll");
    }

    #[test]
    fn fields() {
        let r = make();
        assert_eq!(r.get_team_id(), "team1");
        assert_eq!(r.get_minimum_roll(), 4);
        assert!(r.is_successful());
    }

    #[test]
    fn roll_value() {
        let r = make();
        assert_eq!(r.get_roll(), 5);
    }

    #[test]
    fn unsuccessful_roll_below_minimum() {
        let r = ReportTeamCaptainRoll::new("team2".into(), 4, 3, false);
        assert!(!r.is_successful());
        assert_eq!(r.get_roll(), 3);
        assert_eq!(r.get_team_id(), "team2");
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportTeamCaptainRoll::from_json(&json);
        assert_eq!(restored.team_id, original.team_id);
        assert_eq!(restored.minimum_roll, original.minimum_roll);
        assert_eq!(restored.roll, original.roll);
        assert_eq!(restored.successful, original.successful);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("teamCaptainRoll"));
    }
}
