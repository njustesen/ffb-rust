use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportBribesRoll.java`.
#[derive(Debug, Clone)]
pub struct ReportBribesRoll {
    pub player_id: String,
    pub successful: bool,
    pub roll: i32,
}

impl ReportBribesRoll {
    pub fn new(player_id: String, successful: bool, roll: i32) -> Self {
        Self { player_id, successful, roll }
    }

    pub fn get_player_id(&self) -> &str { &self.player_id }
    pub fn is_successful(&self) -> bool { self.successful }
    pub fn get_roll(&self) -> i32 { self.roll }

    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "playerId": self.player_id,
            "successful": self.successful,
            "roll": self.roll,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            player_id: json["playerId"].as_str().unwrap_or("").to_string(),
            successful: json["successful"].as_bool().unwrap_or(false),
            roll: json["roll"].as_i64().unwrap_or(0) as i32,
        }
    }
}

impl IReport for ReportBribesRoll {
    fn get_id(&self) -> ReportId { ReportId::BRIBES_ROLL }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportBribesRoll {
        ReportBribesRoll::new("p1".into(), true, 4)
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::BRIBES_ROLL);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "bribesRoll");
    }

    #[test]
    fn get_roll() {
        assert_eq!(make().get_roll(), 4);
    }

    #[test]
    fn is_successful() {
        assert!(make().is_successful());
    }

    #[test]
    fn unsuccessful_bribe() {
        let r = ReportBribesRoll::new("p2".into(), false, 2);
        assert!(!r.is_successful());
        assert_eq!(r.get_player_id(), "p2");
        assert_eq!(r.get_roll(), 2);
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportBribesRoll::from_json(&json);
        assert_eq!(restored.player_id, original.player_id);
        assert_eq!(restored.successful, original.successful);
        assert_eq!(restored.roll, original.roll);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("bribesRoll"));
    }
}
