use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportDodgySnackRoll.java`.
#[derive(Debug, Clone)]
pub struct ReportDodgySnackRoll {
    pub roll: i32,
    pub player_id: String,
}

impl ReportDodgySnackRoll {
    pub fn new(roll: i32, player_id: String) -> Self {
        Self { roll, player_id }
    }

    pub fn get_roll(&self) -> i32 { self.roll }
    pub fn get_player_id(&self) -> &str { &self.player_id }

    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "roll": self.roll,
            "playerId": self.player_id,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            roll: json["roll"].as_i64().unwrap_or(0) as i32,
            player_id: json["playerId"].as_str().unwrap_or("").to_string(),
        }
    }
}

impl IReport for ReportDodgySnackRoll {
    fn get_id(&self) -> ReportId { ReportId::DODGY_SNACK_ROLL }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportDodgySnackRoll {
        ReportDodgySnackRoll::new(4, "p1".into())
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::DODGY_SNACK_ROLL);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "dodgySnackRoll");
    }

    #[test]
    fn fields() {
        let r = make();
        assert_eq!(r.get_roll(), 4);
        assert_eq!(r.get_player_id(), "p1");
    }

    #[test]
    fn low_roll() {
        let r = ReportDodgySnackRoll::new(1, "p2".into());
        assert_eq!(r.get_roll(), 1);
        assert_eq!(r.get_player_id(), "p2");
    }

    #[test]
    fn high_roll() {
        let r = ReportDodgySnackRoll::new(6, "p3".into());
        assert_eq!(r.get_roll(), 6);
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportDodgySnackRoll::from_json(&json);
        assert_eq!(restored.roll, original.roll);
        assert_eq!(restored.player_id, original.player_id);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("dodgySnackRoll"));
    }
}
