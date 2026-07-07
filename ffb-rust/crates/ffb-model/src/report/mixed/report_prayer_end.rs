use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportPrayerEnd.java`.
/// `Prayer` is represented as a name string.
#[derive(Debug, Clone)]
pub struct ReportPrayerEnd {
    /// `prayer` — Prayer name string.
    pub prayer: Option<String>,
}

impl ReportPrayerEnd {
    pub fn new(prayer: Option<String>) -> Self {
        Self { prayer }
    }

    pub fn get_prayer(&self) -> Option<&str> { self.prayer.as_deref() }
}

impl IReport for ReportPrayerEnd {
    fn get_id(&self) -> ReportId { ReportId::PRAYER_END }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportPrayerEnd {
        ReportPrayerEnd::new(Some("PRAYER_OF_DEATH".into()))
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::PRAYER_END); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "prayerEnd"); }

    #[test]
    fn get_prayer() { assert_eq!(make().get_prayer(), Some("PRAYER_OF_DEATH")); }

    #[test]
    fn prayer_none() {
        let r = ReportPrayerEnd::new(None);
        assert!(r.get_prayer().is_none());
    }

    #[test]
    fn different_prayer_name() {
        let r = ReportPrayerEnd::new(Some("HAND_OF_GOD".into()));
        assert_eq!(r.get_prayer(), Some("HAND_OF_GOD"));
    }
}
