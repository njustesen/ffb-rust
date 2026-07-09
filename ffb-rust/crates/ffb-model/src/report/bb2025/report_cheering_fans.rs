use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportCheeringFans.java` (bb2025).
#[derive(Debug, Clone)]
pub struct ReportCheeringFans {
    pub team_ids: Vec<String>,
    pub roll_home: i32,
    pub roll_away: i32,
    pub rerolled: Vec<String>,
}

impl ReportCheeringFans {
    pub fn new(team_ids: Vec<String>, roll_home: i32, roll_away: i32, rerolled: Vec<String>) -> Self {
        Self { team_ids, roll_home, roll_away, rerolled }
    }

    pub fn get_team_ids(&self) -> &[String] { &self.team_ids }
    pub fn get_roll_home(&self) -> i32 { self.roll_home }
    pub fn get_roll_away(&self) -> i32 { self.roll_away }
    pub fn get_rerolled(&self) -> &[String] { &self.rerolled }

    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "teamIdsAdditionalAssist": self.team_ids,
            "rollHome": self.roll_home,
            "rollAway": self.roll_away,
            "teamIdsReRolledCheeringFangs": self.rerolled,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            team_ids: json["teamIdsAdditionalAssist"].as_array()
                .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default(),
            roll_home: json["rollHome"].as_i64().unwrap_or(0) as i32,
            roll_away: json["rollAway"].as_i64().unwrap_or(0) as i32,
            rerolled: json["teamIdsReRolledCheeringFangs"].as_array()
                .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default(),
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
        ReportCheeringFans::new(vec!["team1".into()], 4, 2, vec![])
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
        assert_eq!(r.get_roll_home(), 4);
        assert_eq!(r.get_team_ids().len(), 1);
    }

    #[test]
    fn roll_away_and_rerolled() {
        let r = make();
        assert_eq!(r.get_roll_away(), 2);
        assert_eq!(r.get_rerolled().len(), 0);
    }

    #[test]
    fn rerolled_teams() {
        let r = ReportCheeringFans::new(vec!["t1".into(), "t2".into()], 3, 5, vec!["t1".into()]);
        assert_eq!(r.get_team_ids().len(), 2);
        assert_eq!(r.get_rerolled().len(), 1);
        assert_eq!(r.get_rerolled()[0], "t1");
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportCheeringFans::from_json(&json);
        assert_eq!(restored.team_ids, original.team_ids);
        assert_eq!(restored.roll_home, original.roll_home);
        assert_eq!(restored.roll_away, original.roll_away);
        assert_eq!(restored.rerolled, original.rerolled);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("cheeringFans"));
    }
}
