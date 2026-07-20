use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportPlayerEvent.java`.
#[derive(Debug, Clone)]
pub struct ReportPlayerEvent {
    /// `fPlayerId`
    pub player_id: Option<String>,
    pub event_message: Option<String>,
}

impl ReportPlayerEvent {
    pub fn new(player_id: Option<String>, event_message: Option<String>) -> Self {
        Self { player_id, event_message }
    }

    pub fn get_player_id(&self) -> Option<&str> { self.player_id.as_deref() }
    pub fn get_event_message(&self) -> Option<&str> { self.event_message.as_deref() }
}

impl IReport for ReportPlayerEvent {
    fn get_id(&self) -> ReportId { ReportId::PLAYER_EVENT }
}

impl ReportPlayerEvent {
    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "playerId": self.player_id,
            "message": self.event_message,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            player_id: json["playerId"].as_str().map(str::to_string),
            event_message: json["message"].as_str().map(str::to_string),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportPlayerEvent {
        ReportPlayerEvent::new(Some("p1".into()), Some("some event".into()))
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportPlayerEvent::from_json(&json);
        assert_eq!(restored.player_id, original.player_id);
        assert_eq!(restored.event_message, original.event_message);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("playerEvent"));
    }
}
