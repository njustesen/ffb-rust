use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportPettyCash.java`.
#[derive(Debug, Clone)]
pub struct ReportPettyCash {
    pub team_id: String,
    pub gold: i32,
}

impl ReportPettyCash {
    pub fn new(team_id: String, gold: i32) -> Self {
        Self { team_id, gold }
    }

    pub fn get_team_id(&self) -> &str { &self.team_id }
    pub fn get_gold(&self) -> i32 { self.gold }
}

impl IReport for ReportPettyCash {
    fn get_id(&self) -> ReportId { ReportId::PETTY_CASH }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportPettyCash {
        ReportPettyCash::new("team1".into(), 50)
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::PETTY_CASH);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "pettyCash");
    }

    #[test]
    fn fields() {
        let r = make();
        assert_eq!(r.get_team_id(), "team1");
        assert_eq!(r.get_gold(), 50);
    }

    #[test]
    fn zero_gold() {
        let r = ReportPettyCash::new("team2".into(), 0);
        assert_eq!(r.get_gold(), 0);
        assert_eq!(r.get_team_id(), "team2");
    }

    #[test]
    fn large_gold_value() {
        let r = ReportPettyCash::new("team1".into(), 1000000);
        assert_eq!(r.get_gold(), 1000000);
    }
}
