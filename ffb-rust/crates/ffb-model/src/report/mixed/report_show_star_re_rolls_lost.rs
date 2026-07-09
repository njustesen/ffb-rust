use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportShowStarReRollsLost.java`.
#[derive(Debug, Clone)]
pub struct ReportShowStarReRollsLost {
    pub team_id: Option<String>,
    pub amount: i32,
}

impl ReportShowStarReRollsLost {
    pub fn new(team_id: Option<String>, amount: i32) -> Self {
        Self { team_id, amount }
    }

    pub fn get_team_id(&self) -> Option<&str> { self.team_id.as_deref() }
    pub fn get_amount(&self) -> i32 { self.amount }

    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "teamId": self.team_id,
            "rerollShowStarOneDrive": self.amount,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            team_id: json["teamId"].as_str().map(str::to_string),
            amount: json["rerollShowStarOneDrive"].as_i64().unwrap_or(0) as i32,
        }
    }
}

impl IReport for ReportShowStarReRollsLost {
    fn get_id(&self) -> ReportId { ReportId::SHOW_STAR_RE_ROLLS_LOST }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportShowStarReRollsLost {
        ReportShowStarReRollsLost::new(Some("team1".into()), 1)
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::SHOW_STAR_RE_ROLLS_LOST); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "showStarReRollLost"); }

    #[test]
    fn get_amount() { assert_eq!(make().get_amount(), 1); }

    #[test]
    fn get_team_id() { assert_eq!(make().get_team_id(), Some("team1")); }

    #[test]
    fn team_id_none_and_zero_amount() {
        let r = ReportShowStarReRollsLost::new(None, 0);
        assert!(r.get_team_id().is_none());
        assert_eq!(r.get_amount(), 0);
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportShowStarReRollsLost::from_json(&json);
        assert_eq!(restored.team_id, original.team_id);
        assert_eq!(restored.amount, original.amount);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("showStarReRollLost"));
    }
}
