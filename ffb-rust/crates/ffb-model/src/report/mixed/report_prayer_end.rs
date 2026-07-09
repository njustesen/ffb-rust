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

impl ReportPrayerEnd {
    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "prayer": self.prayer,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            prayer: json["prayer"].as_str().map(str::to_string),
        }
    }
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

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportPrayerEnd::from_json(&json);
        assert_eq!(restored.prayer, original.prayer);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("prayerEnd"));
    }
}
