use crate::model::pushback_mode::PushbackMode;
use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportPushback.java`.
#[derive(Debug, Clone)]
pub struct ReportPushback {
    pub defender_id: String,
    pub pushback_mode: PushbackMode,
}

impl ReportPushback {
    pub fn new(defender_id: String, pushback_mode: PushbackMode) -> Self {
        Self { defender_id, pushback_mode }
    }

    pub fn get_defender_id(&self) -> &str { &self.defender_id }
    pub fn get_pushback_mode(&self) -> PushbackMode { self.pushback_mode }

    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "defenderId": self.defender_id,
            "pushbackMode": self.pushback_mode.get_name(),
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            defender_id: json["defenderId"].as_str().unwrap_or("").to_string(),
            pushback_mode: json["pushbackMode"].as_str().and_then(PushbackMode::for_name).unwrap_or(PushbackMode::REGULAR),
        }
    }
}

impl IReport for ReportPushback {
    fn get_id(&self) -> ReportId { ReportId::PUSHBACK }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportPushback {
        ReportPushback::new("def1".into(), PushbackMode::REGULAR)
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::PUSHBACK);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "pushback");
    }

    #[test]
    fn fields() {
        let r = make();
        assert_eq!(r.get_defender_id(), "def1");
        assert_eq!(r.get_pushback_mode(), PushbackMode::REGULAR);
    }

    #[test]
    fn different_defender_id() {
        let r = ReportPushback::new("def2".into(), PushbackMode::SIDE_STEP);
        assert_eq!(r.get_defender_id(), "def2");
    }

    #[test]
    fn side_step_mode() {
        let r = ReportPushback::new("def1".into(), PushbackMode::SIDE_STEP);
        assert_eq!(r.get_pushback_mode(), PushbackMode::SIDE_STEP);
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportPushback::from_json(&json);
        assert_eq!(restored.defender_id, original.defender_id);
        assert_eq!(restored.pushback_mode, original.pushback_mode);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("pushback"));
    }
}
