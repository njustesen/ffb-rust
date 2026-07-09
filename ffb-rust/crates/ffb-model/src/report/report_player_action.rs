use crate::enums::PlayerAction;
use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportPlayerAction.java`.
#[derive(Debug, Clone)]
pub struct ReportPlayerAction {
    pub acting_player_id: String,
    pub player_action: PlayerAction,
}

impl ReportPlayerAction {
    pub fn new(acting_player_id: String, player_action: PlayerAction) -> Self {
        Self { acting_player_id, player_action }
    }

    pub fn get_acting_player_id(&self) -> &str { &self.acting_player_id }
    pub fn get_player_action(&self) -> PlayerAction { self.player_action }

    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "actingPlayerId": self.acting_player_id,
            "playerAction": self.player_action.name(),
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            acting_player_id: json["actingPlayerId"].as_str().unwrap_or("").to_string(),
            player_action: json["playerAction"].as_str().and_then(PlayerAction::from_name).unwrap_or(PlayerAction::Move),
        }
    }
}

impl IReport for ReportPlayerAction {
    fn get_id(&self) -> ReportId { ReportId::PLAYER_ACTION }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportPlayerAction {
        ReportPlayerAction::new("p1".into(), PlayerAction::Move)
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::PLAYER_ACTION);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "playerAction");
    }

    #[test]
    fn fields() {
        let r = make();
        assert_eq!(r.get_acting_player_id(), "p1");
        assert_eq!(r.get_player_action(), PlayerAction::Move);
    }

    #[test]
    fn different_player_id() {
        let r = ReportPlayerAction::new("p99".into(), PlayerAction::Move);
        assert_eq!(r.get_acting_player_id(), "p99");
    }

    #[test]
    fn block_action() {
        let r = ReportPlayerAction::new("p2".into(), PlayerAction::Block);
        assert_eq!(r.get_player_action(), PlayerAction::Block);
        assert_eq!(r.get_acting_player_id(), "p2");
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportPlayerAction::from_json(&json);
        assert_eq!(restored.acting_player_id, original.acting_player_id);
        assert_eq!(restored.player_action, original.player_action);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("playerAction"));
    }
}
