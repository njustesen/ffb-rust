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
}
