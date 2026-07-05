use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportShowStarReRollsLost.java`.
#[derive(Debug, Clone)]
pub struct ReportShowStarReRollsLost {
    pub team_id: Option<String>,
    pub amount: i32,
}

impl ReportShowStarReRollsLost {
    pub fn new(team_id: Option<String>, amount: i32) -> Self {
        Self { team_id, amount }
    }

    pub fn get_team_id(&self) -> Option<&str> { self.team_id.as_deref() }
    pub fn get_amount(&self) -> i32 { self.amount }
}

impl IReport for ReportShowStarReRollsLost {
    fn get_id(&self) -> ReportId { ReportId::SHOW_STAR_RE_ROLLS_LOST }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportShowStarReRollsLost {
        ReportShowStarReRollsLost::new(Some("team1".into()), 1)
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::SHOW_STAR_RE_ROLLS_LOST); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "showStarReRollLost"); }

    #[test]
    fn get_amount() { assert_eq!(make().get_amount(), 1); }
}
