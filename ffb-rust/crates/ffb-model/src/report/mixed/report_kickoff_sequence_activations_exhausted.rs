use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportKickoffSequenceActivationsExhausted.java`.
#[derive(Debug, Clone)]
pub struct ReportKickoffSequenceActivationsExhausted {
    pub limit_reached: bool,
}

impl ReportKickoffSequenceActivationsExhausted {
    pub fn new(limit_reached: bool) -> Self {
        Self { limit_reached }
    }

    pub fn is_limit_reached(&self) -> bool { self.limit_reached }
}

impl IReport for ReportKickoffSequenceActivationsExhausted {
    fn get_id(&self) -> ReportId { ReportId::KICKOFF_SEQUENCE_ACTIVATIONS_EXHAUSTED }
}

impl ReportKickoffSequenceActivationsExhausted {
    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "limitReached": self.limit_reached,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            limit_reached: json["limitReached"].as_bool().unwrap_or(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportKickoffSequenceActivationsExhausted {
        ReportKickoffSequenceActivationsExhausted::new(true)
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::KICKOFF_SEQUENCE_ACTIVATIONS_EXHAUSTED); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "kickoffSequenceActivationsExhausted"); }

    #[test]
    fn is_limit_reached() { assert!(make().is_limit_reached()); }

    #[test]
    fn not_limit_reached() {
        let r = ReportKickoffSequenceActivationsExhausted::new(false);
        assert!(!r.is_limit_reached());
    }

    #[test]
    fn get_name_matches() {
        assert_eq!(make().get_name(), "kickoffSequenceActivationsExhausted");
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportKickoffSequenceActivationsExhausted::from_json(&json);
        assert_eq!(restored.limit_reached, original.limit_reached);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("kickoffSequenceActivationsExhausted"));
    }
}
