use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportBrilliantCoachingReRollsLost.java`.
#[derive(Debug, Clone)]
pub struct ReportBrilliantCoachingReRollsLost {
    pub team_id: Option<String>,
    pub amount: i32,
}

impl ReportBrilliantCoachingReRollsLost {
    pub fn new(team_id: Option<String>, amount: i32) -> Self {
        Self { team_id, amount }
    }

    pub fn get_team_id(&self) -> Option<&str> { self.team_id.as_deref() }
    pub fn get_amount(&self) -> i32 { self.amount }
}

impl IReport for ReportBrilliantCoachingReRollsLost {
    fn get_id(&self) -> ReportId { ReportId::BRILLIANT_COACHING_RE_ROLLS_LOST }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportBrilliantCoachingReRollsLost {
        ReportBrilliantCoachingReRollsLost::new(Some("team1".into()), 2)
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::BRILLIANT_COACHING_RE_ROLLS_LOST); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "brilliantCoachingReRoll"); }

    #[test]
    fn get_amount() { assert_eq!(make().get_amount(), 2); }
}
