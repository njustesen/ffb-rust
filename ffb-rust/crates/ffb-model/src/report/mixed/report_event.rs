use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportEvent.java`.
#[derive(Debug, Clone)]
pub struct ReportEvent {
    pub event_message: Option<String>,
}

impl ReportEvent {
    pub fn new(event_message: Option<String>) -> Self {
        Self { event_message }
    }

    pub fn get_event_message(&self) -> Option<&str> { self.event_message.as_deref() }
}

impl IReport for ReportEvent {
    fn get_id(&self) -> ReportId { ReportId::EVENT }
}

impl ReportEvent {
    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "message": self.event_message,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            event_message: json["message"].as_str().map(str::to_string),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportEvent {
        ReportEvent::new(Some("something happened".into()))
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::EVENT); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "event"); }

    #[test]
    fn get_event_message() { assert_eq!(make().get_event_message(), Some("something happened")); }

    #[test]
    fn none_event_message() {
        let r = ReportEvent::new(None);
        assert_eq!(r.get_event_message(), None);
    }

    #[test]
    fn different_event_message() {
        let r = ReportEvent::new(Some("touchdown".into()));
        assert_eq!(r.get_event_message(), Some("touchdown"));
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportEvent::from_json(&json);
        assert_eq!(restored.event_message, original.event_message);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("event"));
    }
}
