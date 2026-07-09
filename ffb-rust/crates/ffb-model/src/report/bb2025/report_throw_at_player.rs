use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportThrowAtPlayer.java`.
#[derive(Debug, Clone)]
pub struct ReportThrowAtPlayer {
    pub player_id: String,
    pub roll: i32,
    pub successful: bool,
}

impl ReportThrowAtPlayer {
    pub fn new(player_id: String, roll: i32, successful: bool) -> Self {
        Self { player_id, roll, successful }
    }

    pub fn get_player_id(&self) -> &str { &self.player_id }
    pub fn get_roll(&self) -> i32 { self.roll }
    pub fn is_successful(&self) -> bool { self.successful }

    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "playerId": self.player_id,
            "roll": self.roll,
            "successful": self.successful,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            player_id: json["playerId"].as_str().unwrap_or("").to_string(),
            roll: json["roll"].as_i64().unwrap_or(0) as i32,
            successful: json["successful"].as_bool().unwrap_or(false),
        }
    }
}

impl IReport for ReportThrowAtPlayer {
    fn get_id(&self) -> ReportId { ReportId::THROW_AT_PLAYER }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportThrowAtPlayer {
        ReportThrowAtPlayer::new("p1".into(), 4, true)
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::THROW_AT_PLAYER);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "throwAtPlayer");
    }

    #[test]
    fn fields() {
        let r = make();
        assert_eq!(r.get_player_id(), "p1");
        assert_eq!(r.get_roll(), 4);
        assert!(r.is_successful());
    }

    #[test]
    fn unsuccessful() {
        let r = ReportThrowAtPlayer::new("p2".into(), 2, false);
        assert!(!r.is_successful());
        assert_eq!(r.get_roll(), 2);
        assert_eq!(r.get_player_id(), "p2");
    }

    #[test]
    fn high_roll_successful() {
        let r = ReportThrowAtPlayer::new("p3".into(), 6, true);
        assert_eq!(r.get_roll(), 6);
        assert!(r.is_successful());
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportThrowAtPlayer::from_json(&json);
        assert_eq!(restored.player_id, original.player_id);
        assert_eq!(restored.roll, original.roll);
        assert_eq!(restored.successful, original.successful);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("throwAtPlayer"));
    }
}
