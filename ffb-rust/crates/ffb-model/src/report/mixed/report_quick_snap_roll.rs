use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportQuickSnapRoll.java`.
#[derive(Debug, Clone)]
pub struct ReportQuickSnapRoll {
    pub team_id: Option<String>,
    pub amount: i32,
    pub roll: i32,
}

impl ReportQuickSnapRoll {
    pub fn new(team_id: Option<String>, roll: i32, amount: i32) -> Self {
        Self { team_id, amount, roll }
    }

    pub fn get_team_id(&self) -> Option<&str> { self.team_id.as_deref() }
    pub fn get_amount(&self) -> i32 { self.amount }
    pub fn get_roll(&self) -> i32 { self.roll }
}

impl IReport for ReportQuickSnapRoll {
    fn get_id(&self) -> ReportId { ReportId::QUICK_SNAP_ROLL }
}

impl ReportQuickSnapRoll {
    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "nrOfPlayers": self.amount,
            "teamId": self.team_id,
            "roll": self.roll,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            team_id: json["teamId"].as_str().map(str::to_string),
            amount: json["nrOfPlayers"].as_i64().unwrap_or(0) as i32,
            roll: json["roll"].as_i64().unwrap_or(0) as i32,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportQuickSnapRoll {
        ReportQuickSnapRoll::new(Some("team1".into()), 2, 3)
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::QUICK_SNAP_ROLL); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "quickSnapRoll"); }

    #[test]
    fn get_roll() { assert_eq!(make().get_roll(), 2); }

    #[test]
    fn get_amount() { assert_eq!(make().get_amount(), 3); }
    #[test]
    fn get_name_is_nonempty() {
        assert!(!make().get_name().is_empty());
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportQuickSnapRoll::from_json(&json);
        assert_eq!(restored.team_id, original.team_id);
        assert_eq!(restored.amount, original.amount);
        assert_eq!(restored.roll, original.roll);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("quickSnapRoll"));
    }
}
