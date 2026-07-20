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
