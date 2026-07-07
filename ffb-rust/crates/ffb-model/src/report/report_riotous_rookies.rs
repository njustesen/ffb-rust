use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportRiotousRookies.java`.
#[derive(Debug, Clone)]
pub struct ReportRiotousRookies {
    pub roll: Vec<i32>,
    pub amount: i32,
    pub team_id: String,
}

impl ReportRiotousRookies {
    pub fn new(roll: Vec<i32>, amount: i32, team_id: String) -> Self {
        Self { roll, amount, team_id }
    }

    pub fn get_roll(&self) -> &[i32] { &self.roll }
    pub fn get_amount(&self) -> i32 { self.amount }
    pub fn get_team_id(&self) -> &str { &self.team_id }
}

impl IReport for ReportRiotousRookies {
    fn get_id(&self) -> ReportId { ReportId::RIOTOUS_ROOKIES }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportRiotousRookies {
        ReportRiotousRookies::new(vec![2, 3], 1, "team1".into())
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::RIOTOUS_ROOKIES);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "riotousRookies");
    }

    #[test]
    fn fields() {
        let r = make();
        assert_eq!(r.get_roll(), &[2, 3]);
        assert_eq!(r.get_amount(), 1);
        assert_eq!(r.get_team_id(), "team1");
    }

    #[test]
    fn empty_roll() {
        let r = ReportRiotousRookies::new(vec![], 0, "team2".into());
        assert_eq!(r.get_roll().len(), 0);
        assert_eq!(r.get_amount(), 0);
    }

    #[test]
    fn different_team_id() {
        let r = ReportRiotousRookies::new(vec![5], 2, "team2".into());
        assert_eq!(r.get_team_id(), "team2");
        assert_eq!(r.get_amount(), 2);
    }
}
