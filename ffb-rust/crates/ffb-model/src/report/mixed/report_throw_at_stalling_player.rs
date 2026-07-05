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
}
