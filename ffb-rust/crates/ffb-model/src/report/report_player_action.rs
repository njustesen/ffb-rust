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
}
