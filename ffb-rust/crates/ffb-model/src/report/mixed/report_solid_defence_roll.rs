use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportSolidDefenceRoll.java`.
#[derive(Debug, Clone)]
pub struct ReportSolidDefenceRoll {
    pub team_id: Option<String>,
    pub amount: i32,
    pub roll: i32,
}

impl ReportSolidDefenceRoll {
    pub fn new(team_id: Option<String>, roll: i32, amount: i32) -> Self {
        Self { team_id, amount, roll }
    }

    pub fn get_team_id(&self) -> Option<&str> { self.team_id.as_deref() }
    pub fn get_amount(&self) -> i32 { self.amount }
    pub fn get_roll(&self) -> i32 { self.roll }
}

impl IReport for ReportSolidDefenceRoll {
    fn get_id(&self) -> ReportId { ReportId::SOLID_DEFENCE_ROLL }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportSolidDefenceRoll {
        ReportSolidDefenceRoll::new(Some("team1".into()), 3, 2)
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::SOLID_DEFENCE_ROLL); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "solidDefenceRoll"); }

    #[test]
    fn get_roll() { assert_eq!(make().get_roll(), 3); }

    #[test]
    fn get_amount() { assert_eq!(make().get_amount(), 2); }
    #[test]
    fn get_name_is_nonempty() {
        assert!(!make().get_name().is_empty());
    }
}
