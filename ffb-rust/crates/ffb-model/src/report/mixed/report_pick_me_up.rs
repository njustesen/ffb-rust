use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportPickMeUp.java`.
#[derive(Debug, Clone)]
pub struct ReportPickMeUp {
    pub player_id: Option<String>,
    pub success: bool,
    pub roll: i32,
}

impl ReportPickMeUp {
    pub fn new(player_id: Option<String>, roll: i32, success: bool) -> Self {
        Self { player_id, success, roll }
    }

    pub fn get_player_id(&self) -> Option<&str> { self.player_id.as_deref() }
    pub fn is_success(&self) -> bool { self.success }
    pub fn get_roll(&self) -> i32 { self.roll }
}

impl IReport for ReportPickMeUp {
    fn get_id(&self) -> ReportId { ReportId::PICK_ME_UP }
}

impl ReportPickMeUp {
    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "successful": self.success,
            "playerId": self.player_id,
            "roll": self.roll,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            player_id: json["playerId"].as_str().map(str::to_string),
            success: json["successful"].as_bool().unwrap_or(false),
            roll: json["roll"].as_i64().unwrap_or(0) as i32,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportPickMeUp {
        ReportPickMeUp::new(Some("p1".into()), 5, true)
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::PICK_ME_UP); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "pickMeUp"); }

    #[test]
    fn get_roll() { assert_eq!(make().get_roll(), 5); }

    #[test]
    fn is_success() { assert!(make().is_success()); }
    #[test]
    fn get_name_is_nonempty() {
        assert!(!make().get_name().is_empty());
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportPickMeUp::from_json(&json);
        assert_eq!(restored.player_id, original.player_id);
        assert_eq!(restored.success, original.success);
        assert_eq!(restored.roll, original.roll);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("pickMeUp"));
    }
}
