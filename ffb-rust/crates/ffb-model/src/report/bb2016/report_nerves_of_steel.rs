use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportNervesOfSteel.java`.
#[derive(Debug, Clone)]
pub struct ReportNervesOfSteel {
    pub player_id: String,
    pub ball_action: String,
}

impl ReportNervesOfSteel {
    pub fn new(player_id: String, ball_action: String) -> Self {
        Self { player_id, ball_action }
    }

    pub fn get_player_id(&self) -> &str { &self.player_id }
    pub fn get_ball_action(&self) -> &str { &self.ball_action }

    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "playerId": self.player_id,
            "ballAction": self.ball_action,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            player_id: json["playerId"].as_str().unwrap_or("").to_string(),
            ball_action: json["ballAction"].as_str().unwrap_or("").to_string(),
        }
    }
}

impl IReport for ReportNervesOfSteel {
    fn get_id(&self) -> ReportId { ReportId::NERVES_OF_STEEL }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportNervesOfSteel {
        ReportNervesOfSteel::new("p1".into(), "pass".into())
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::NERVES_OF_STEEL);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "nervesOfSteel");
    }

    #[test]
    fn fields() {
        let r = make();
        assert_eq!(r.get_player_id(), "p1");
        assert_eq!(r.get_ball_action(), "pass");
    }

    #[test]
    fn different_ball_action() {
        let r = ReportNervesOfSteel::new("p2".into(), "catch".into());
        assert_eq!(r.get_ball_action(), "catch");
    }

    #[test]
    fn different_player_id() {
        let r = ReportNervesOfSteel::new("p99".into(), "handoff".into());
        assert_eq!(r.get_player_id(), "p99");
        assert_eq!(r.get_ball_action(), "handoff");
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportNervesOfSteel::from_json(&json);
        assert_eq!(restored.player_id, original.player_id);
        assert_eq!(restored.ball_action, original.ball_action);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("nervesOfSteel"));
    }
}
