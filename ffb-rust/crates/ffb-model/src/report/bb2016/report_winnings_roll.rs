use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportWinningsRoll.java`.
#[derive(Debug, Clone)]
pub struct ReportWinningsRoll {
    pub winnings_roll_home: i32,
    pub winnings_home: i32,
    pub winnings_roll_away: i32,
    pub winnings_away: i32,
}

impl ReportWinningsRoll {
    pub fn new(
        winnings_roll_home: i32,
        winnings_home: i32,
        winnings_roll_away: i32,
        winnings_away: i32,
    ) -> Self {
        Self { winnings_roll_home, winnings_home, winnings_roll_away, winnings_away }
    }

    pub fn get_winnings_roll_home(&self) -> i32 { self.winnings_roll_home }
    pub fn get_winnings_home(&self) -> i32 { self.winnings_home }
    pub fn get_winnings_roll_away(&self) -> i32 { self.winnings_roll_away }
    pub fn get_winnings_away(&self) -> i32 { self.winnings_away }
}

impl IReport for ReportWinningsRoll {
    fn get_id(&self) -> ReportId { ReportId::WINNINGS_ROLL }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportWinningsRoll {
        ReportWinningsRoll::new(4, 40000, 2, 20000)
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::WINNINGS_ROLL);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "winningsRoll");
    }

    #[test]
    fn fields() {
        let r = make();
        assert_eq!(r.get_winnings_roll_home(), 4);
        assert_eq!(r.get_winnings_home(), 40000);
        assert_eq!(r.get_winnings_away(), 20000);
    }

    #[test]
    fn roll_away_stored() {
        let r = make();
        assert_eq!(r.get_winnings_roll_away(), 2);
    }

    #[test]
    fn zero_winnings() {
        let r = ReportWinningsRoll::new(0, 0, 0, 0);
        assert_eq!(r.get_winnings_home(), 0);
        assert_eq!(r.get_winnings_away(), 0);
    }
}
