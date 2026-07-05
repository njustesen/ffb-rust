use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportFreePettyCash.java`.
#[derive(Debug, Clone)]
pub struct ReportFreePettyCash {
    pub team_id: Option<String>,
    pub gold: i32,
}

impl ReportFreePettyCash {
    pub fn new(team_id: Option<String>, gold: i32) -> Self {
        Self { team_id, gold }
    }

    pub fn get_team_id(&self) -> Option<&str> { self.team_id.as_deref() }
    pub fn get_gold(&self) -> i32 { self.gold }
}

impl IReport for ReportFreePettyCash {
    fn get_id(&self) -> ReportId { ReportId::FREE_PETTY_CASH }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportFreePettyCash {
        ReportFreePettyCash::new(Some("team1".into()), 50000)
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::FREE_PETTY_CASH); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "freePettyCash"); }

    #[test]
    fn get_gold() { assert_eq!(make().get_gold(), 50000); }
}
