use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportCheeringFans.java` (bb2025).
#[derive(Debug, Clone)]
pub struct ReportCheeringFans {
    pub team_ids: Vec<String>,
    pub roll_home: i32,
    pub roll_away: i32,
    pub rerolled: Vec<String>,
}

impl ReportCheeringFans {
    pub fn new(team_ids: Vec<String>, roll_home: i32, roll_away: i32, rerolled: Vec<String>) -> Self {
        Self { team_ids, roll_home, roll_away, rerolled }
    }

    pub fn get_team_ids(&self) -> &[String] { &self.team_ids }
    pub fn get_roll_home(&self) -> i32 { self.roll_home }
    pub fn get_roll_away(&self) -> i32 { self.roll_away }
    pub fn get_rerolled(&self) -> &[String] { &self.rerolled }
}

impl IReport for ReportCheeringFans {
    fn get_id(&self) -> ReportId { ReportId::KICKOFF_CHEERING_FANS }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportCheeringFans {
        ReportCheeringFans::new(vec!["team1".into()], 4, 2, vec![])
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::KICKOFF_CHEERING_FANS);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "cheeringFans");
    }

    #[test]
    fn fields() {
        let r = make();
        assert_eq!(r.get_roll_home(), 4);
        assert_eq!(r.get_team_ids().len(), 1);
    }

    #[test]
    fn roll_away_and_rerolled() {
        let r = make();
        assert_eq!(r.get_roll_away(), 2);
        assert_eq!(r.get_rerolled().len(), 0);
    }

    #[test]
    fn rerolled_teams() {
        let r = ReportCheeringFans::new(vec!["t1".into(), "t2".into()], 3, 5, vec!["t1".into()]);
        assert_eq!(r.get_team_ids().len(), 2);
        assert_eq!(r.get_rerolled().len(), 1);
        assert_eq!(r.get_rerolled()[0], "t1");
    }
}
