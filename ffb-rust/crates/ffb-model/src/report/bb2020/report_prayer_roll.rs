use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportPrayerRoll.java` (bb2020).
#[derive(Debug, Clone)]
pub struct ReportPrayerRoll {
    pub roll: i32,
}

impl ReportPrayerRoll {
    pub fn new(roll: i32) -> Self {
        Self { roll }
    }

    pub fn get_roll(&self) -> i32 { self.roll }
}

impl IReport for ReportPrayerRoll {
    fn get_id(&self) -> ReportId { ReportId::PRAYER_ROLL }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_id() {
        assert_eq!(ReportPrayerRoll::new(3).get_id(), ReportId::PRAYER_ROLL);
    }

    #[test]
    fn get_name() {
        assert_eq!(ReportPrayerRoll::new(3).get_name(), "prayerRoll");
    }

    #[test]
    fn fields() {
        assert_eq!(ReportPrayerRoll::new(5).get_roll(), 5);
    }
}
