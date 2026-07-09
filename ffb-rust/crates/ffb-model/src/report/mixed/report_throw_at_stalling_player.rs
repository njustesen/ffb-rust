use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportThrowAtStallingPlayer.java`.
#[derive(Debug, Clone)]
pub struct ReportThrowAtStallingPlayer {
    /// `fPlayerId`
    pub player_id: Option<String>,
    pub roll: i32,
    pub successful: bool,
}

impl ReportThrowAtStallingPlayer {
    pub fn new(player_id: Option<String>, roll: i32, successful: bool) -> Self {
        Self { player_id, roll, successful }
    }

    pub fn get_player_id(&self) -> Option<&str> { self.player_id.as_deref() }
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
            player_id: json["playerId"].as_str().map(str::to_string),
            roll: json["roll"].as_i64().unwrap_or(0) as i32,
            successful: json["successful"].as_bool().unwrap_or(false),
        }
    }
}

impl IReport for ReportThrowAtStallingPlayer {
    fn get_id(&self) -> ReportId { ReportId::THROW_AT_STALLING_PLAYER }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportThrowAtStallingPlayer {
        ReportThrowAtStallingPlayer::new(Some("p1".into()), 5, true)
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::THROW_AT_STALLING_PLAYER); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "throwAtStallingPlayer"); }

    #[test]
    fn get_roll() { assert_eq!(make().get_roll(), 5); }

    #[test]
    fn is_successful() { assert!(make().is_successful()); }
    #[test]
    fn get_name_is_nonempty() {
        assert!(!make().get_name().is_empty());
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportThrowAtStallingPlayer::from_json(&json);
        assert_eq!(restored.player_id, original.player_id);
        assert_eq!(restored.roll, original.roll);
        assert_eq!(restored.successful, original.successful);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("throwAtStallingPlayer"));
    }
}
