use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportPrayerWasted.java`.
#[derive(Debug, Clone)]
pub struct ReportPrayerWasted {
    /// `name` — prayer name.
    pub prayer_name: Option<String>,
    pub player_id: Option<String>,
}

impl ReportPrayerWasted {
    pub fn new(prayer_name: Option<String>, player_id: Option<String>) -> Self {
        Self { prayer_name, player_id }
    }

    pub fn get_prayer_name(&self) -> Option<&str> { self.prayer_name.as_deref() }
    pub fn get_player_id(&self) -> Option<&str> { self.player_id.as_deref() }
}

impl IReport for ReportPrayerWasted {
    fn get_id(&self) -> ReportId { ReportId::PRAYER_WASTED }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportPrayerWasted {
        ReportPrayerWasted::new(Some("PRAYER_OF_DEATH".into()), Some("p1".into()))
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::PRAYER_WASTED); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "prayerWasted"); }

    #[test]
    fn get_prayer_name() { assert_eq!(make().get_prayer_name(), Some("PRAYER_OF_DEATH")); }

    #[test]
    fn get_player_id() { assert_eq!(make().get_player_id(), Some("p1")); }
}
