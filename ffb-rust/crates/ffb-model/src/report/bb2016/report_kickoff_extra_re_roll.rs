use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;
use crate::enums::KickoffResult;

/// 1:1 translation of `ReportKickoffExtraReRoll.java`.
#[derive(Debug, Clone)]
pub struct ReportKickoffExtraReRoll {
    pub kickoff_result: KickoffResult,
    pub roll_home: i32,
    pub home_gains_re_roll: bool,
    pub roll_away: i32,
    pub away_gains_re_roll: bool,
}

impl ReportKickoffExtraReRoll {
    pub fn new(
        kickoff_result: KickoffResult,
        roll_home: i32,
        home_gains_re_roll: bool,
        roll_away: i32,
        away_gains_re_roll: bool,
    ) -> Self {
        Self { kickoff_result, roll_home, home_gains_re_roll, roll_away, away_gains_re_roll }
    }

    pub fn get_kickoff_result(&self) -> KickoffResult { self.kickoff_result }
    pub fn get_roll_home(&self) -> i32 { self.roll_home }
    pub fn is_home_gains_re_roll(&self) -> bool { self.home_gains_re_roll }
    pub fn get_roll_away(&self) -> i32 { self.roll_away }
    pub fn is_away_gains_re_roll(&self) -> bool { self.away_gains_re_roll }
}

impl IReport for ReportKickoffExtraReRoll {
    fn get_id(&self) -> ReportId { ReportId::KICKOFF_EXTRA_RE_ROLL }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportKickoffExtraReRoll {
        ReportKickoffExtraReRoll::new(KickoffResult::BrilliantCoaching, 3, true, 2, false)
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::KICKOFF_EXTRA_RE_ROLL);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "extraReRoll");
    }

    #[test]
    fn fields() {
        let r = make();
        assert_eq!(r.get_roll_home(), 3);
        assert!(r.is_home_gains_re_roll());
        assert!(!r.is_away_gains_re_roll());
    }
}
