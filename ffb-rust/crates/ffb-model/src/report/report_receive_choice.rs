use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportReceiveChoice.java`.
#[derive(Debug, Clone)]
pub struct ReportReceiveChoice {
    pub team_id: String,
    pub receive_choice: bool,
}

impl ReportReceiveChoice {
    pub fn new(team_id: String, receive_choice: bool) -> Self {
        Self { team_id, receive_choice }
    }

    pub fn get_team_id(&self) -> &str { &self.team_id }
    pub fn is_receive_choice(&self) -> bool { self.receive_choice }

    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "teamId": self.team_id,
            "receiveChoice": self.receive_choice,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            team_id: json["teamId"].as_str().unwrap_or("").to_string(),
            receive_choice: json["receiveChoice"].as_bool().unwrap_or(false),
        }
    }
}

impl IReport for ReportReceiveChoice {
    fn get_id(&self) -> ReportId { ReportId::RECEIVE_CHOICE }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportReceiveChoice {
        ReportReceiveChoice::new("team1".into(), true)
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::RECEIVE_CHOICE);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "receiveChoice");
    }

    #[test]
    fn fields() {
        let r = make();
        assert_eq!(r.get_team_id(), "team1");
        assert!(r.is_receive_choice());
    }

    #[test]
    fn different_team_id() {
        let r = ReportReceiveChoice::new("team2".into(), false);
        assert_eq!(r.get_team_id(), "team2");
    }

    #[test]
    fn receive_choice_false() {
        let r = ReportReceiveChoice::new("team1".into(), false);
        assert!(!r.is_receive_choice());
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportReceiveChoice::from_json(&json);
        assert_eq!(restored.team_id, original.team_id);
        assert_eq!(restored.receive_choice, original.receive_choice);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("receiveChoice"));
    }
}
