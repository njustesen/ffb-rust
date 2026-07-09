use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportFreePettyCash.java`.
#[derive(Debug, Clone)]
pub struct ReportFreePettyCash {
    pub team_id: Option<String>,
    pub gold: i32,
}

impl ReportFreePettyCash {
    pub fn new(team_id: Option<String>, gold: i32) -> Self {
        Self { team_id, gold }
    }

    pub fn get_team_id(&self) -> Option<&str> { self.team_id.as_deref() }
    pub fn get_gold(&self) -> i32 { self.gold }
}

impl IReport for ReportFreePettyCash {
    fn get_id(&self) -> ReportId { ReportId::FREE_PETTY_CASH }
}

impl ReportFreePettyCash {
    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "teamId": self.team_id,
            "gold": self.gold,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            team_id: json["teamId"].as_str().map(str::to_string),
            gold: json["gold"].as_i64().unwrap_or(0) as i32,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportFreePettyCash {
        ReportFreePettyCash::new(Some("team1".into()), 50000)
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::FREE_PETTY_CASH); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "freePettyCash"); }

    #[test]
    fn get_gold() { assert_eq!(make().get_gold(), 50000); }

    #[test]
    fn get_team_id() { assert_eq!(make().get_team_id(), Some("team1")); }

    #[test]
    fn none_team_id() {
        let r = ReportFreePettyCash::new(None, 10000);
        assert_eq!(r.get_team_id(), None);
        assert_eq!(r.get_gold(), 10000);
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportFreePettyCash::from_json(&json);
        assert_eq!(restored.team_id, original.team_id);
        assert_eq!(restored.gold, original.gold);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("freePettyCash"));
    }
}
