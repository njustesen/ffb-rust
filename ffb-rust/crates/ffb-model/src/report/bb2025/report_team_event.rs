use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportTeamEvent.java`.
#[derive(Debug, Clone)]
pub struct ReportTeamEvent {
    pub team_id: String,
    pub event_message: String,
}

impl ReportTeamEvent {
    pub fn new(team_id: String, event_message: String) -> Self {
        Self { team_id, event_message }
    }

    pub fn get_team_id(&self) -> &str { &self.team_id }
    pub fn get_event_message(&self) -> &str { &self.event_message }

    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "teamId": self.team_id,
            "message": self.event_message,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            team_id: json["teamId"].as_str().unwrap_or("").to_string(),
            event_message: json["message"].as_str().unwrap_or("").to_string(),
        }
    }
}

impl IReport for ReportTeamEvent {
    fn get_id(&self) -> ReportId { ReportId::TEAM_EVENT }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportTeamEvent {
        ReportTeamEvent::new("team1".into(), "Player banned!".into())
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportTeamEvent::from_json(&json);
        assert_eq!(restored.team_id, original.team_id);
        assert_eq!(restored.event_message, original.event_message);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("teamEvent"));
    }
}
