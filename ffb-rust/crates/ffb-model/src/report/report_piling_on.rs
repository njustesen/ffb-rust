use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportPilingOn.java`.
#[derive(Debug, Clone)]
pub struct ReportPilingOn {
    pub player_id: String,
    pub used: bool,
    pub re_roll_injury: bool,
}

impl ReportPilingOn {
    pub fn new(player_id: String, used: bool, re_roll_injury: bool) -> Self {
        Self { player_id, used, re_roll_injury }
    }

    pub fn get_player_id(&self) -> &str { &self.player_id }
    pub fn is_used(&self) -> bool { self.used }
    pub fn is_re_roll_injury(&self) -> bool { self.re_roll_injury }

    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "playerId": self.player_id,
            "used": self.used,
            "reRollInjury": self.re_roll_injury,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            player_id: json["playerId"].as_str().unwrap_or("").to_string(),
            used: json["used"].as_bool().unwrap_or(false),
            re_roll_injury: json["reRollInjury"].as_bool().unwrap_or(false),
        }
    }
}

impl IReport for ReportPilingOn {
    fn get_id(&self) -> ReportId { ReportId::PILING_ON }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportPilingOn {
        ReportPilingOn::new("p1".into(), true, false)
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::PILING_ON);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "pilingOn");
    }

    #[test]
    fn fields() {
        let r = make();
        assert_eq!(r.get_player_id(), "p1");
        assert!(r.is_used());
        assert!(!r.is_re_roll_injury());
    }

    #[test]
    fn not_used() {
        let r = ReportPilingOn::new("p2".into(), false, false);
        assert!(!r.is_used());
        assert_eq!(r.get_player_id(), "p2");
    }

    #[test]
    fn re_roll_injury_flag() {
        let r = ReportPilingOn::new("p3".into(), true, true);
        assert!(r.is_re_roll_injury());
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportPilingOn::from_json(&json);
        assert_eq!(restored.player_id, original.player_id);
        assert_eq!(restored.used, original.used);
        assert_eq!(restored.re_roll_injury, original.re_roll_injury);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("pilingOn"));
    }
}
