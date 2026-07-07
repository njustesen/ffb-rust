use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportPumpUpTheCrowdReRollsLost.java`.
#[derive(Debug, Clone)]
pub struct ReportPumpUpTheCrowdReRollsLost {
    pub team_id: Option<String>,
    pub amount: i32,
}

impl ReportPumpUpTheCrowdReRollsLost {
    pub fn new(team_id: Option<String>, amount: i32) -> Self {
        Self { team_id, amount }
    }

    pub fn get_team_id(&self) -> Option<&str> { self.team_id.as_deref() }
    pub fn get_amount(&self) -> i32 { self.amount }
}

impl IReport for ReportPumpUpTheCrowdReRollsLost {
    fn get_id(&self) -> ReportId { ReportId::PUMP_UP_THE_CROWD_RE_ROLLS_LOST }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportPumpUpTheCrowdReRollsLost {
        ReportPumpUpTheCrowdReRollsLost::new(Some("team1".into()), 2)
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::PUMP_UP_THE_CROWD_RE_ROLLS_LOST); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "pumpUpTheCrowdReRollLost"); }

    #[test]
    fn get_amount() { assert_eq!(make().get_amount(), 2); }

    #[test]
    fn get_team_id() { assert_eq!(make().get_team_id(), Some("team1")); }

    #[test]
    fn team_id_none() {
        let r = ReportPumpUpTheCrowdReRollsLost::new(None, 0);
        assert!(r.get_team_id().is_none());
        assert_eq!(r.get_amount(), 0);
    }
}
