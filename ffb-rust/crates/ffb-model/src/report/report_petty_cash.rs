use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportPettyCash.java`.
#[derive(Debug, Clone)]
pub struct ReportPettyCash {
    pub team_id: String,
    pub gold: i32,
}

impl ReportPettyCash {
    pub fn new(team_id: String, gold: i32) -> Self {
        Self { team_id, gold }
    }

    pub fn get_team_id(&self) -> &str { &self.team_id }
    pub fn get_gold(&self) -> i32 { self.gold }

    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "teamId": self.team_id,
            "gold": self.gold,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            team_id: json["teamId"].as_str().unwrap_or("").to_string(),
            gold: json["gold"].as_i64().unwrap_or(0) as i32,
        }
    }
}

impl IReport for ReportPettyCash {
    fn get_id(&self) -> ReportId { ReportId::PETTY_CASH }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportPettyCash {
        ReportPettyCash::new("team1".into(), 50)
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::PETTY_CASH);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "pettyCash");
    }

    #[test]
    fn fields() {
        let r = make();
        assert_eq!(r.get_team_id(), "team1");
        assert_eq!(r.get_gold(), 50);
    }

    #[test]
    fn zero_gold() {
        let r = ReportPettyCash::new("team2".into(), 0);
        assert_eq!(r.get_gold(), 0);
        assert_eq!(r.get_team_id(), "team2");
    }

    #[test]
    fn large_gold_value() {
        let r = ReportPettyCash::new("team1".into(), 1000000);
        assert_eq!(r.get_gold(), 1000000);
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportPettyCash::from_json(&json);
        assert_eq!(restored.team_id, original.team_id);
        assert_eq!(restored.gold, original.gold);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("pettyCash"));
    }
}
