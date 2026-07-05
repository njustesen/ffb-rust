use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportPlayerEvent.java`.
#[derive(Debug, Clone)]
pub struct ReportPlayerEvent {
    /// `fPlayerId`
    pub player_id: Option<String>,
    pub event_message: Option<String>,
}

impl ReportPlayerEvent {
    pub fn new(player_id: Option<String>, event_message: Option<String>) -> Self {
        Self { player_id, event_message }
    }

    pub fn get_player_id(&self) -> Option<&str> { self.player_id.as_deref() }
    pub fn get_event_message(&self) -> Option<&str> { self.event_message.as_deref() }
}

impl IReport for ReportPlayerEvent {
    fn get_id(&self) -> ReportId { ReportId::PLAYER_EVENT }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportPlayerEvent {
        ReportPlayerEvent::new(Some("p1".into()), Some("some event".into()))
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::PLAYER_EVENT); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "playerEvent"); }

    #[test]
    fn get_event_message() { assert_eq!(make().get_event_message(), Some("some event")); }
}
