use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportBiasedRef.java`.
#[derive(Debug, Clone)]
pub struct ReportBiasedRef {
    pub foul_spotted: bool,
    pub roll: i32,
}

impl ReportBiasedRef {
    pub fn new(foul_spotted: bool, roll: i32) -> Self {
        Self { foul_spotted, roll }
    }

    pub fn is_foul_spotted(&self) -> bool { self.foul_spotted }
    pub fn get_roll(&self) -> i32 { self.roll }
}

impl IReport for ReportBiasedRef {
    fn get_id(&self) -> ReportId { ReportId::BIASED_REF }
}

impl ReportBiasedRef {
    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "foulingPlayerBanned": self.foul_spotted,
            "roll": self.roll,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            foul_spotted: json["foulingPlayerBanned"].as_bool().unwrap_or(false),
            roll: json["roll"].as_i64().unwrap_or(0) as i32,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportBiasedRef {
        ReportBiasedRef::new(true, 3)
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::BIASED_REF); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "biasedRef"); }

    #[test]
    fn get_roll() { assert_eq!(make().get_roll(), 3); }

    #[test]
    fn is_foul_spotted() { assert!(make().is_foul_spotted()); }

    #[test]
    fn not_foul_spotted() {
        let r = ReportBiasedRef::new(false, 1);
        assert!(!r.is_foul_spotted());
        assert_eq!(r.get_roll(), 1);
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportBiasedRef::from_json(&json);
        assert_eq!(restored.foul_spotted, original.foul_spotted);
        assert_eq!(restored.roll, original.roll);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("biasedRef"));
    }
}
