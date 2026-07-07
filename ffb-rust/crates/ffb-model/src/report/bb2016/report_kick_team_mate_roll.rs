use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportKickTeamMateRoll.java`.
#[derive(Debug, Clone)]
pub struct ReportKickTeamMateRoll {
    pub kicking_player_id: String,
    pub kicked_player_id: String,
    pub kick_distance: i32,
    pub successful: bool,
    pub re_rolled: bool,
    pub roll: Vec<i32>,
}

impl ReportKickTeamMateRoll {
    pub fn new(
        kicking_player_id: String,
        kicked_player_id: String,
        successful: bool,
        roll: Vec<i32>,
        re_rolled: bool,
        kick_distance: i32,
    ) -> Self {
        Self { kicking_player_id, kicked_player_id, kick_distance, successful, re_rolled, roll }
    }

    pub fn get_kicking_player_id(&self) -> &str { &self.kicking_player_id }
    pub fn get_kicked_player_id(&self) -> &str { &self.kicked_player_id }
    pub fn get_kick_distance(&self) -> i32 { self.kick_distance }
    pub fn is_successful(&self) -> bool { self.successful }
    pub fn is_re_rolled(&self) -> bool { self.re_rolled }
    pub fn get_roll(&self) -> &[i32] { &self.roll }
}

impl IReport for ReportKickTeamMateRoll {
    fn get_id(&self) -> ReportId { ReportId::KICK_TEAM_MATE_ROLL }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportKickTeamMateRoll {
        ReportKickTeamMateRoll::new("kicker".into(), "kicked".into(), true, vec![3, 4], false, 3)
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::KICK_TEAM_MATE_ROLL);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "kickTeamMateRoll");
    }

    #[test]
    fn fields() {
        let r = make();
        assert_eq!(r.get_kicking_player_id(), "kicker");
        assert_eq!(r.get_kick_distance(), 3);
        assert!(r.is_successful());
    }

    #[test]
    fn kicked_player_and_roll() {
        let r = make();
        assert_eq!(r.get_kicked_player_id(), "kicked");
        assert_eq!(r.get_roll(), &[3, 4]);
    }

    #[test]
    fn rerolled_unsuccessful() {
        let r = ReportKickTeamMateRoll::new("k1".into(), "k2".into(), false, vec![1], true, 2);
        assert!(!r.is_successful());
        assert!(r.is_re_rolled());
        assert_eq!(r.get_kick_distance(), 2);
    }
}
