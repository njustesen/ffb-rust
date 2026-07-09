use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportOldPro.java`.
#[derive(Debug, Clone)]
pub struct ReportOldPro {
    pub player_id: Option<String>,
    pub old_value: i32,
    pub new_value: i32,
    pub self_inflicted: bool,
}

impl ReportOldPro {
    pub fn new(player_id: Option<String>, old_value: i32, new_value: i32, self_inflicted: bool) -> Self {
        Self { player_id, old_value, new_value, self_inflicted }
    }

    pub fn get_player_id(&self) -> Option<&str> { self.player_id.as_deref() }
    pub fn get_old_value(&self) -> i32 { self.old_value }
    pub fn get_new_value(&self) -> i32 { self.new_value }
    pub fn is_self_inflicted(&self) -> bool { self.self_inflicted }
}

impl IReport for ReportOldPro {
    fn get_id(&self) -> ReportId { ReportId::OLD_PRO }
}

impl ReportOldPro {
    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "playerId": self.player_id,
            "oldRoll": self.old_value,
            "roll": self.new_value,
            "selfInflicted": self.self_inflicted,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            player_id: json["playerId"].as_str().map(str::to_string),
            old_value: json["oldRoll"].as_i64().unwrap_or(0) as i32,
            new_value: json["roll"].as_i64().unwrap_or(0) as i32,
            self_inflicted: json["selfInflicted"].as_bool().unwrap_or(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportOldPro {
        ReportOldPro::new(Some("p1".into()), 3, 2, false)
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::OLD_PRO); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "oldPro"); }

    #[test]
    fn get_new_value() { assert_eq!(make().get_new_value(), 2); }

    #[test]
    fn get_old_value_and_player_id() {
        let r = make();
        assert_eq!(r.get_old_value(), 3);
        assert_eq!(r.get_player_id(), Some("p1"));
    }

    #[test]
    fn is_self_inflicted() {
        let r = ReportOldPro::new(None, 1, 0, true);
        assert!(r.is_self_inflicted());
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportOldPro::from_json(&json);
        assert_eq!(restored.player_id, original.player_id);
        assert_eq!(restored.old_value, original.old_value);
        assert_eq!(restored.new_value, original.new_value);
        assert_eq!(restored.self_inflicted, original.self_inflicted);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("oldPro"));
    }
}
