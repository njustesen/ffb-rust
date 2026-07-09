use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportNervesOfSteel.java`.
#[derive(Debug, Clone)]
pub struct ReportNervesOfSteel {
    pub player_id: Option<String>,
    pub ball_action: Option<String>,
    pub bomb: bool,
}

impl ReportNervesOfSteel {
    pub fn new(player_id: Option<String>, ball_action: Option<String>, bomb: bool) -> Self {
        Self { player_id, ball_action, bomb }
    }

    pub fn get_player_id(&self) -> Option<&str> { self.player_id.as_deref() }
    pub fn get_ball_action(&self) -> Option<&str> { self.ball_action.as_deref() }
    pub fn is_bomb(&self) -> bool { self.bomb }
}

impl IReport for ReportNervesOfSteel {
    fn get_id(&self) -> ReportId { ReportId::NERVES_OF_STEEL }
}

impl ReportNervesOfSteel {
    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "playerId": self.player_id,
            "ballAction": self.ball_action,
            "bomb": self.bomb,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            player_id: json["playerId"].as_str().map(str::to_string),
            ball_action: json["ballAction"].as_str().map(str::to_string),
            bomb: json["bomb"].as_bool().unwrap_or(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportNervesOfSteel {
        ReportNervesOfSteel::new(Some("p1".into()), Some("PASS".into()), false)
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::NERVES_OF_STEEL); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "nervesOfSteel"); }

    #[test]
    fn get_ball_action() { assert_eq!(make().get_ball_action(), Some("PASS")); }

    #[test]
    fn get_player_id() { assert_eq!(make().get_player_id(), Some("p1")); }

    #[test]
    fn is_bomb_true() {
        let r = ReportNervesOfSteel::new(None, None, true);
        assert!(r.is_bomb());
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportNervesOfSteel::from_json(&json);
        assert_eq!(restored.player_id, original.player_id);
        assert_eq!(restored.ball_action, original.ball_action);
        assert_eq!(restored.bomb, original.bomb);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("nervesOfSteel"));
    }
}
