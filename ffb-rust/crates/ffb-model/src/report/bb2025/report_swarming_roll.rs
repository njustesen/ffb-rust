use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportSwarmingRoll.java`.
#[derive(Debug, Clone)]
pub struct ReportSwarmingRoll {
    pub team_id: String,
    pub roll: i32,
}

impl ReportSwarmingRoll {
    pub fn new(team_id: String, roll: i32) -> Self {
        Self { team_id, roll }
    }

    pub fn get_team_id(&self) -> &str { &self.team_id }
    pub fn get_roll(&self) -> i32 { self.roll }

    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "teamId": self.team_id,
            "swarmingPlayerRoll": self.roll,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            team_id: json["teamId"].as_str().unwrap_or("").to_string(),
            roll: json["swarmingPlayerRoll"].as_i64().unwrap_or(0) as i32,
        }
    }
}

impl IReport for ReportSwarmingRoll {
    fn get_id(&self) -> ReportId { ReportId::SWARMING_PLAYERS_ROLL }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_id() {
        assert_eq!(ReportSwarmingRoll::new("t1".into(), 2).get_id(), ReportId::SWARMING_PLAYERS_ROLL);
    }

    #[test]
    fn get_name() {
        assert_eq!(ReportSwarmingRoll::new("t1".into(), 2).get_name(), "swarmingPlayersRoll");
    }

    #[test]
    fn fields() {
        let r = ReportSwarmingRoll::new("t1".into(), 2);
        assert_eq!(r.get_team_id(), "t1");
        assert_eq!(r.get_roll(), 2);
    }

    #[test]
    fn high_roll() {
        let r = ReportSwarmingRoll::new("t2".into(), 6);
        assert_eq!(r.get_roll(), 6);
        assert_eq!(r.get_team_id(), "t2");
    }

    #[test]
    fn low_roll() {
        let r = ReportSwarmingRoll::new("t3".into(), 1);
        assert_eq!(r.get_roll(), 1);
    }

    #[test]
    fn serialization_round_trip() {
        let original = ReportSwarmingRoll::new("t1".into(), 2);
        let json = original.to_json_value();
        let restored = ReportSwarmingRoll::from_json(&json);
        assert_eq!(restored.team_id, original.team_id);
        assert_eq!(restored.roll, original.roll);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = ReportSwarmingRoll::new("t1".into(), 2).to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("swarmingPlayersRoll"));
    }
}
