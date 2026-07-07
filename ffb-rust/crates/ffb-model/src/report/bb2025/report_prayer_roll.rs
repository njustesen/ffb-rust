use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportPrayerRoll.java` (bb2025).
#[derive(Debug, Clone)]
pub struct ReportPrayerRoll {
    pub team_name: String,
    pub roll: i32,
    pub home_team: bool,
}

impl ReportPrayerRoll {
    pub fn new(team_name: String, roll: i32, home_team: bool) -> Self {
        Self { team_name, roll, home_team }
    }

    pub fn get_team_name(&self) -> &str { &self.team_name }
    pub fn get_roll(&self) -> i32 { self.roll }
    pub fn is_home_team(&self) -> bool { self.home_team }
}

impl IReport for ReportPrayerRoll {
    fn get_id(&self) -> ReportId { ReportId::PRAYER_ROLL }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportPrayerRoll {
        ReportPrayerRoll::new("Home Ultras".into(), 5, true)
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::PRAYER_ROLL);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "prayerRoll");
    }

    #[test]
    fn fields() {
        let r = make();
        assert_eq!(r.get_team_name(), "Home Ultras");
        assert_eq!(r.get_roll(), 5);
        assert!(r.is_home_team());
    }

    #[test]
    fn away_team() {
        let r = ReportPrayerRoll::new("Away Raiders".into(), 3, false);
        assert!(!r.is_home_team());
        assert_eq!(r.get_team_name(), "Away Raiders");
    }

    #[test]
    fn roll_value() {
        let r = ReportPrayerRoll::new("Team".into(), 6, true);
        assert_eq!(r.get_roll(), 6);
    }
}
