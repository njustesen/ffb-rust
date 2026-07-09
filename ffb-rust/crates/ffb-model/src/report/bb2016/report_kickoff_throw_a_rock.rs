use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportKickoffThrowARock.java`.
#[derive(Debug, Clone)]
pub struct ReportKickoffThrowARock {
    pub roll_home: i32,
    pub roll_away: i32,
    pub players_hit: Vec<String>,
}

impl ReportKickoffThrowARock {
    pub fn new(roll_home: i32, roll_away: i32, players_hit: Vec<String>) -> Self {
        Self { roll_home, roll_away, players_hit }
    }

    pub fn get_roll_home(&self) -> i32 { self.roll_home }
    pub fn get_roll_away(&self) -> i32 { self.roll_away }
    pub fn get_players_hit(&self) -> &[String] { &self.players_hit }

    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "rollHome": self.roll_home,
            "rollAway": self.roll_away,
            "playerIdsHit": self.players_hit,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            roll_home: json["rollHome"].as_i64().unwrap_or(0) as i32,
            roll_away: json["rollAway"].as_i64().unwrap_or(0) as i32,
            players_hit: json["playerIdsHit"].as_array().map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_string)).collect()).unwrap_or_default(),
        }
    }
}

impl IReport for ReportKickoffThrowARock {
    fn get_id(&self) -> ReportId { ReportId::KICKOFF_THROW_A_ROCK }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportKickoffThrowARock {
        ReportKickoffThrowARock::new(4, 2, vec!["p1".into(), "p2".into()])
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::KICKOFF_THROW_A_ROCK);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "kickoffThrowARock");
    }

    #[test]
    fn fields() {
        let r = make();
        assert_eq!(r.get_roll_home(), 4);
        assert_eq!(r.get_players_hit().len(), 2);
    }

    #[test]
    fn roll_away_stored() {
        let r = make();
        assert_eq!(r.get_roll_away(), 2);
    }

    #[test]
    fn players_hit_contents() {
        let r = make();
        assert_eq!(r.get_players_hit()[0], "p1");
        assert_eq!(r.get_players_hit()[1], "p2");
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportKickoffThrowARock::from_json(&json);
        assert_eq!(restored.roll_home, original.roll_home);
        assert_eq!(restored.roll_away, original.roll_away);
        assert_eq!(restored.players_hit, original.players_hit);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("kickoffThrowARock"));
    }
}
