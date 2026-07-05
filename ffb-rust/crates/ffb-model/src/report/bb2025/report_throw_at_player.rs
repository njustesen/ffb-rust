use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportThrowAtPlayer.java`.
#[derive(Debug, Clone)]
pub struct ReportThrowAtPlayer {
    pub player_id: String,
    pub roll: i32,
    pub successful: bool,
}

impl ReportThrowAtPlayer {
    pub fn new(player_id: String, roll: i32, successful: bool) -> Self {
        Self { player_id, roll, successful }
    }

    pub fn get_player_id(&self) -> &str { &self.player_id }
    pub fn get_roll(&self) -> i32 { self.roll }
    pub fn is_successful(&self) -> bool { self.successful }
}

impl IReport for ReportThrowAtPlayer {
    fn get_id(&self) -> ReportId { ReportId::THROW_AT_PLAYER }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportThrowAtPlayer {
        ReportThrowAtPlayer::new("p1".into(), 4, true)
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::THROW_AT_PLAYER);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "throwAtPlayer");
    }

    #[test]
    fn fields() {
        let r = make();
        assert_eq!(r.get_player_id(), "p1");
        assert_eq!(r.get_roll(), 4);
        assert!(r.is_successful());
    }
}
