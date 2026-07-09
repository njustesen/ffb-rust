use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportPuntDistance.java`.
#[derive(Debug, Clone)]
pub struct ReportPuntDistance {
    pub roll: i32,
    pub out_of_bounds: bool,
}

impl ReportPuntDistance {
    pub fn new(roll: i32, out_of_bounds: bool) -> Self {
        Self { roll, out_of_bounds }
    }

    pub fn get_roll(&self) -> i32 { self.roll }
    pub fn is_out_of_bounds(&self) -> bool { self.out_of_bounds }

    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "roll": self.roll,
            "outOfBounds": self.out_of_bounds,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            roll: json["roll"].as_i64().unwrap_or(0) as i32,
            out_of_bounds: json["outOfBounds"].as_bool().unwrap_or(false),
        }
    }
}

impl IReport for ReportPuntDistance {
    fn get_id(&self) -> ReportId { ReportId::PUNT_DISTANCE_ROLL }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_id() {
        assert_eq!(ReportPuntDistance::new(4, false).get_id(), ReportId::PUNT_DISTANCE_ROLL);
    }

    #[test]
    fn get_name() {
        assert_eq!(ReportPuntDistance::new(4, false).get_name(), "puntDistanceRoll");
    }

    #[test]
    fn fields() {
        let r = ReportPuntDistance::new(4, true);
        assert_eq!(r.get_roll(), 4);
        assert!(r.is_out_of_bounds());
    }

    #[test]
    fn not_out_of_bounds() {
        let r = ReportPuntDistance::new(3, false);
        assert!(!r.is_out_of_bounds());
        assert_eq!(r.get_roll(), 3);
    }

    #[test]
    fn max_roll() {
        let r = ReportPuntDistance::new(6, false);
        assert_eq!(r.get_roll(), 6);
    }

    #[test]
    fn serialization_round_trip() {
        let original = ReportPuntDistance::new(4, false);
        let json = original.to_json_value();
        let restored = ReportPuntDistance::from_json(&json);
        assert_eq!(restored.roll, original.roll);
        assert_eq!(restored.out_of_bounds, original.out_of_bounds);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = ReportPuntDistance::new(4, false).to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("puntDistanceRoll"));
    }
}
