use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportSelectGazeTarget.java`.
#[derive(Debug, Clone)]
pub struct ReportSelectGazeTarget {
    pub attacker: Option<String>,
    pub defender: Option<String>,
}

impl ReportSelectGazeTarget {
    pub fn new(attacker: Option<String>, defender: Option<String>) -> Self {
        Self { attacker, defender }
    }

    pub fn get_attacker(&self) -> Option<&str> { self.attacker.as_deref() }
    pub fn get_defender(&self) -> Option<&str> { self.defender.as_deref() }

    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "attackerId": self.attacker,
            "defenderId": self.defender,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            attacker: json["attackerId"].as_str().map(str::to_string),
            defender: json["defenderId"].as_str().map(str::to_string),
        }
    }
}

impl IReport for ReportSelectGazeTarget {
    fn get_id(&self) -> ReportId { ReportId::SELECT_GAZE_TARGET }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportSelectGazeTarget {
        ReportSelectGazeTarget::new(Some("a1".into()), Some("d1".into()))
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::SELECT_GAZE_TARGET); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "selectGazeTarget"); }

    #[test]
    fn get_attacker() { assert_eq!(make().get_attacker(), Some("a1")); }

    #[test]
    fn get_defender() { assert_eq!(make().get_defender(), Some("d1")); }
    #[test]
    fn get_name_is_nonempty() {
        assert!(!make().get_name().is_empty());
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportSelectGazeTarget::from_json(&json);
        assert_eq!(restored.attacker, original.attacker);
        assert_eq!(restored.defender, original.defender);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("selectGazeTarget"));
    }
}
