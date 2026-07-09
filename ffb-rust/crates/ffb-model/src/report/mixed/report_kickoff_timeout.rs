use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportKickoffTimeout.java`.
#[derive(Debug, Clone)]
pub struct ReportKickoffTimeout {
    pub turn_modifier: i32,
    pub turn_number: i32,
}

impl ReportKickoffTimeout {
    pub fn new(turn_modifier: i32, turn_number: i32) -> Self {
        Self { turn_modifier, turn_number }
    }

    pub fn get_turn_modifier(&self) -> i32 { self.turn_modifier }
    pub fn get_turn_number(&self) -> i32 { self.turn_number }
}

impl IReport for ReportKickoffTimeout {
    fn get_id(&self) -> ReportId { ReportId::KICKOFF_TIMEOUT }
}

impl ReportKickoffTimeout {
    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "turnModifier": self.turn_modifier,
            "turnNr": self.turn_number,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            turn_modifier: json["turnModifier"].as_i64().unwrap_or(0) as i32,
            turn_number: json["turnNr"].as_i64().unwrap_or(0) as i32,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportKickoffTimeout {
        ReportKickoffTimeout::new(1, 4)
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::KICKOFF_TIMEOUT); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "kickoffTimeout"); }

    #[test]
    fn get_turn_number() { assert_eq!(make().get_turn_number(), 4); }

    #[test]
    fn get_turn_modifier() { assert_eq!(make().get_turn_modifier(), 1); }

    #[test]
    fn negative_modifier() {
        let r = ReportKickoffTimeout::new(-1, 8);
        assert_eq!(r.get_turn_modifier(), -1);
        assert_eq!(r.get_turn_number(), 8);
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportKickoffTimeout::from_json(&json);
        assert_eq!(restored.turn_modifier, original.turn_modifier);
        assert_eq!(restored.turn_number, original.turn_number);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("kickoffTimeout"));
    }
}
