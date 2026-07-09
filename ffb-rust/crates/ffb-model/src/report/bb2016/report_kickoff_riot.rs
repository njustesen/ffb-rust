use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportKickoffRiot.java`.
#[derive(Debug, Clone)]
pub struct ReportKickoffRiot {
    pub roll: i32,
    pub turn_modifier: i32,
}

impl ReportKickoffRiot {
    pub fn new(roll: i32, turn_modifier: i32) -> Self {
        Self { roll, turn_modifier }
    }

    pub fn get_roll(&self) -> i32 { self.roll }
    pub fn get_turn_modifier(&self) -> i32 { self.turn_modifier }

    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "roll": self.roll,
            "turnModifier": self.turn_modifier,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            roll: json["roll"].as_i64().unwrap_or(0) as i32,
            turn_modifier: json["turnModifier"].as_i64().unwrap_or(0) as i32,
        }
    }
}

impl IReport for ReportKickoffRiot {
    fn get_id(&self) -> ReportId { ReportId::KICKOFF_RIOT }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportKickoffRiot {
        ReportKickoffRiot::new(3, -1)
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::KICKOFF_RIOT);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "kickoffRiot");
    }

    #[test]
    fn fields() {
        let r = make();
        assert_eq!(r.get_roll(), 3);
        assert_eq!(r.get_turn_modifier(), -1);
    }

    #[test]
    fn positive_turn_modifier() {
        let r = ReportKickoffRiot::new(5, 1);
        assert_eq!(r.get_roll(), 5);
        assert_eq!(r.get_turn_modifier(), 1);
    }

    #[test]
    fn zero_modifier() {
        let r = ReportKickoffRiot::new(4, 0);
        assert_eq!(r.get_turn_modifier(), 0);
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportKickoffRiot::from_json(&json);
        assert_eq!(restored.roll, original.roll);
        assert_eq!(restored.turn_modifier, original.turn_modifier);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("kickoffRiot"));
    }
}
