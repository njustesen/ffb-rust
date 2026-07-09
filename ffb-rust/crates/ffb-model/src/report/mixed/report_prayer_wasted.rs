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

impl ReportPrayerWasted {
    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "playerId": self.player_id,
            "name": self.prayer_name,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            prayer_name: json["name"].as_str().map(str::to_string),
            player_id: json["playerId"].as_str().map(str::to_string),
        }
    }
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
    #[test]
    fn get_name_is_nonempty() {
        assert!(!make().get_name().is_empty());
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportPrayerWasted::from_json(&json);
        assert_eq!(restored.prayer_name, original.prayer_name);
        assert_eq!(restored.player_id, original.player_id);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("prayerWasted"));
    }
}
