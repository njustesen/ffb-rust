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
}
