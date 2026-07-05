use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportBlitzRoll.java`.
#[derive(Debug, Clone)]
pub struct ReportBlitzRoll {
    pub team_id: Option<String>,
    pub amount: i32,
    pub roll: i32,
}

impl ReportBlitzRoll {
    pub fn new(team_id: Option<String>, amount: i32, roll: i32) -> Self {
        Self { team_id, amount, roll }
    }

    pub fn get_team_id(&self) -> Option<&str> { self.team_id.as_deref() }
    pub fn get_amount(&self) -> i32 { self.amount }
    pub fn get_roll(&self) -> i32 { self.roll }
}

impl IReport for ReportBlitzRoll {
    fn get_id(&self) -> ReportId { ReportId::BLITZ_ROLL }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportBlitzRoll {
        ReportBlitzRoll::new(Some("team1".into()), 2, 4)
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::BLITZ_ROLL); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "blitzRoll"); }

    #[test]
    fn get_roll() { assert_eq!(make().get_roll(), 4); }
}
