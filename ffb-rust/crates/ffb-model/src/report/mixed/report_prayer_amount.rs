use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportPrayerAmount.java`.
#[derive(Debug, Clone)]
pub struct ReportPrayerAmount {
    pub tv_home: i32,
    pub tv_away: i32,
    pub prayer_amount: i32,
    pub home_team_receives_prayers: bool,
}

impl ReportPrayerAmount {
    pub fn new(tv_home: i32, tv_away: i32, prayer_amount: i32, home_team_receives_prayers: bool) -> Self {
        Self { tv_home, tv_away, prayer_amount, home_team_receives_prayers }
    }

    pub fn get_tv_home(&self) -> i32 { self.tv_home }
    pub fn get_tv_away(&self) -> i32 { self.tv_away }
    pub fn get_prayer_amount(&self) -> i32 { self.prayer_amount }
    pub fn is_home_team_receives_prayers(&self) -> bool { self.home_team_receives_prayers }
}

impl IReport for ReportPrayerAmount {
    fn get_id(&self) -> ReportId { ReportId::PRAYER_AMOUNT }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportPrayerAmount {
        ReportPrayerAmount::new(1000, 800, 3, true)
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::PRAYER_AMOUNT); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "prayerAmount"); }

    #[test]
    fn get_prayer_amount() { assert_eq!(make().get_prayer_amount(), 3); }

    #[test]
    fn is_home_team_receives_prayers() { assert!(make().is_home_team_receives_prayers()); }
}
