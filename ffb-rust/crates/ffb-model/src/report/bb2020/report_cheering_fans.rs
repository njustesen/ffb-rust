use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportCheeringFans.java` (bb2020).
#[derive(Debug, Clone)]
pub struct ReportCheeringFans {
    pub team_id: String,
    pub prayer_available: bool,
    pub roll_home: i32,
    pub roll_away: i32,
}

impl ReportCheeringFans {
    pub fn new(team_id: String, prayer_available: bool, roll_home: i32, roll_away: i32) -> Self {
        Self { team_id, prayer_available, roll_home, roll_away }
    }

    pub fn get_team_id(&self) -> &str { &self.team_id }
    pub fn is_prayer_available(&self) -> bool { self.prayer_available }
    pub fn get_roll_home(&self) -> i32 { self.roll_home }
    pub fn get_roll_away(&self) -> i32 { self.roll_away }

    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "teamId": self.team_id,
            "prayerAvailable": self.prayer_available,
            "rollHome": self.roll_home,
            "rollAway": self.roll_away,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            team_id: json["teamId"].as_str().unwrap_or("").to_string(),
            prayer_available: json["prayerAvailable"].as_bool().unwrap_or(false),
            roll_home: json["rollHome"].as_i64().unwrap_or(0) as i32,
            roll_away: json["rollAway"].as_i64().unwrap_or(0) as i32,
        }
    }
}

impl IReport for ReportCheeringFans {
    fn get_id(&self) -> ReportId { ReportId::KICKOFF_CHEERING_FANS }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportCheeringFans {
        ReportCheeringFans::new("team1".into(), true, 4, 2)
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::KICKOFF_CHEERING_FANS);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "cheeringFans");
    }

    #[test]
    fn fields() {
        let r = make();
        assert!(r.is_prayer_available());
        assert_eq!(r.get_roll_home(), 4);
        assert_eq!(r.get_roll_away(), 2);
    }

    #[test]
    fn prayer_not_available() {
        let r = ReportCheeringFans::new("team2".into(), false, 3, 5);
        assert!(!r.is_prayer_available());
        assert_eq!(r.get_team_id(), "team2");
    }

    #[test]
    fn roll_away_value() {
        let r = ReportCheeringFans::new("team3".into(), true, 1, 6);
        assert_eq!(r.get_roll_away(), 6);
        assert_eq!(r.get_roll_home(), 1);
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportCheeringFans::from_json(&json);
        assert_eq!(restored.team_id, original.team_id);
        assert_eq!(restored.prayer_available, original.prayer_available);
        assert_eq!(restored.roll_home, original.roll_home);
        assert_eq!(restored.roll_away, original.roll_away);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("cheeringFans"));
    }
}
