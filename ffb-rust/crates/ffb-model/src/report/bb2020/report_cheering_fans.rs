use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportCheeringFans.java` (bb2020).
#[derive(Debug, Clone)]
pub struct ReportCheeringFans {
    pub team_id: String,
    pub prayer_available: bool,
    pub roll_home: i32,
    pub roll_away: i32,
}

impl ReportCheeringFans {
    pub fn new(team_id: String, prayer_available: bool, roll_home: i32, roll_away: i32) -> Self {
        Self { team_id, prayer_available, roll_home, roll_away }
    }

    pub fn get_team_id(&self) -> &str { &self.team_id }
    pub fn is_prayer_available(&self) -> bool { self.prayer_available }
    pub fn get_roll_home(&self) -> i32 { self.roll_home }
    pub fn get_roll_away(&self) -> i32 { self.roll_away }
}

impl IReport for ReportCheeringFans {
    fn get_id(&self) -> ReportId { ReportId::KICKOFF_CHEERING_FANS }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportCheeringFans {
        ReportCheeringFans::new("team1".into(), true, 4, 2)
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
        assert!(r.is_prayer_available());
        assert_eq!(r.get_roll_home(), 4);
        assert_eq!(r.get_roll_away(), 2);
    }

    #[test]
    fn prayer_not_available() {
        let r = ReportCheeringFans::new("team2".into(), false, 3, 5);
        assert!(!r.is_prayer_available());
        assert_eq!(r.get_team_id(), "team2");
    }

    #[test]
    fn roll_away_value() {
        let r = ReportCheeringFans::new("team3".into(), true, 1, 6);
        assert_eq!(r.get_roll_away(), 6);
        assert_eq!(r.get_roll_home(), 1);
    }
}
