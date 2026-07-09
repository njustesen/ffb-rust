use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportThenIStartedBlastin.java`.
#[derive(Debug, Clone)]
pub struct ReportThenIStartedBlastin {
    pub player_id: Option<String>,
    pub target_player_id: Option<String>,
    pub roll: i32,
    pub success: bool,
    pub fumble: bool,
}

impl ReportThenIStartedBlastin {
    pub fn new(
        player_id: Option<String>,
        target_player_id: Option<String>,
        roll: i32,
        success: bool,
        fumble: bool,
    ) -> Self {
        Self { player_id, target_player_id, roll, success, fumble }
    }

    pub fn get_player_id(&self) -> Option<&str> { self.player_id.as_deref() }
    pub fn get_target_player_id(&self) -> Option<&str> { self.target_player_id.as_deref() }
    pub fn get_roll(&self) -> i32 { self.roll }
    pub fn is_success(&self) -> bool { self.success }
    pub fn is_fumble(&self) -> bool { self.fumble }

    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "playerId": self.player_id,
            "roll": self.roll,
            "targetPlayerId": self.target_player_id,
            "successful": self.success,
            "fumble": self.fumble,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            player_id: json["playerId"].as_str().map(str::to_string),
            target_player_id: json["targetPlayerId"].as_str().map(str::to_string),
            roll: json["roll"].as_i64().unwrap_or(0) as i32,
            success: json["successful"].as_bool().unwrap_or(false),
            fumble: json["fumble"].as_bool().unwrap_or(false),
        }
    }
}

impl IReport for ReportThenIStartedBlastin {
    fn get_id(&self) -> ReportId { ReportId::THEN_I_STARTED_BLASTIN }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportThenIStartedBlastin {
        ReportThenIStartedBlastin::new(Some("p1".into()), Some("t1".into()), 4, true, false)
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::THEN_I_STARTED_BLASTIN); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "thenIStartedBlastin"); }

    #[test]
    fn get_roll() { assert_eq!(make().get_roll(), 4); }

    #[test]
    fn is_success() { assert!(make().is_success()); }

    #[test]
    fn is_fumble() { assert!(!make().is_fumble()); }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportThenIStartedBlastin::from_json(&json);
        assert_eq!(restored.player_id, original.player_id);
        assert_eq!(restored.target_player_id, original.target_player_id);
        assert_eq!(restored.roll, original.roll);
        assert_eq!(restored.success, original.success);
        assert_eq!(restored.fumble, original.fumble);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("thenIStartedBlastin"));
    }
}
