use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportWinnings.java`.
#[derive(Debug, Clone)]
pub struct ReportWinnings {
    pub winnings_home: i32,
    pub winnings_away: i32,
}

impl ReportWinnings {
    pub fn new(winnings_home: i32, winnings_away: i32) -> Self {
        Self { winnings_home, winnings_away }
    }

    pub fn get_winnings_home(&self) -> i32 { self.winnings_home }
    pub fn get_winnings_away(&self) -> i32 { self.winnings_away }
}

impl IReport for ReportWinnings {
    fn get_id(&self) -> ReportId { ReportId::WINNINGS }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportWinnings {
        ReportWinnings::new(50000, 30000)
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::WINNINGS); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "winnings"); }

    #[test]
    fn get_winnings_home() { assert_eq!(make().get_winnings_home(), 50000); }

    #[test]
    fn get_winnings_away() { assert_eq!(make().get_winnings_away(), 30000); }
}
