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

    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "roll": self.roll,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            roll: json["roll"].as_i64().unwrap_or(0) as i32,
        }
    }
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

    #[test]
    fn roll_minimum() {
        assert_eq!(ReportPrayerRoll::new(1).get_roll(), 1);
    }

    #[test]
    fn roll_maximum() {
        assert_eq!(ReportPrayerRoll::new(6).get_roll(), 6);
    }

    #[test]
    fn serialization_round_trip() {
        let original = ReportPrayerRoll::new(3);
        let json = original.to_json_value();
        let restored = ReportPrayerRoll::from_json(&json);
        assert_eq!(restored.roll, original.roll);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = ReportPrayerRoll::new(3).to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("prayerRoll"));
    }
}
