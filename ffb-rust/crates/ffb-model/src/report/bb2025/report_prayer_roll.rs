use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportPrayerRoll.java` (bb2025).
#[derive(Debug, Clone)]
pub struct ReportPrayerRoll {
    pub team_name: String,
    pub roll: i32,
    pub home_team: bool,
}

impl ReportPrayerRoll {
    pub fn new(team_name: String, roll: i32, home_team: bool) -> Self {
        Self { team_name, roll, home_team }
    }

    pub fn get_team_name(&self) -> &str { &self.team_name }
    pub fn get_roll(&self) -> i32 { self.roll }
    pub fn is_home_team(&self) -> bool { self.home_team }

    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "teamName": self.team_name,
            "roll": self.roll,
            "homeTeam": self.home_team,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            team_name: json["teamName"].as_str().unwrap_or("").to_string(),
            roll: json["roll"].as_i64().unwrap_or(0) as i32,
            home_team: json["homeTeam"].as_bool().unwrap_or(false),
        }
    }
}

impl IReport for ReportPrayerRoll {
    fn get_id(&self) -> ReportId { ReportId::PRAYER_ROLL }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportPrayerRoll {
        ReportPrayerRoll::new("Home Ultras".into(), 5, true)
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::PRAYER_ROLL);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "prayerRoll");
    }

    #[test]
    fn fields() {
        let r = make();
        assert_eq!(r.get_team_name(), "Home Ultras");
        assert_eq!(r.get_roll(), 5);
        assert!(r.is_home_team());
    }

    #[test]
    fn away_team() {
        let r = ReportPrayerRoll::new("Away Raiders".into(), 3, false);
        assert!(!r.is_home_team());
        assert_eq!(r.get_team_name(), "Away Raiders");
    }

    #[test]
    fn roll_value() {
        let r = ReportPrayerRoll::new("Team".into(), 6, true);
        assert_eq!(r.get_roll(), 6);
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportPrayerRoll::from_json(&json);
        assert_eq!(restored.team_name, original.team_name);
        assert_eq!(restored.roll, original.roll);
        assert_eq!(restored.home_team, original.home_team);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("prayerRoll"));
    }
}
