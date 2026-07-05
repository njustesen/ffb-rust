use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportTeamEvent.java`.
#[derive(Debug, Clone)]
pub struct ReportTeamEvent {
    pub team_id: String,
    pub event_message: String,
}

impl ReportTeamEvent {
    pub fn new(team_id: String, event_message: String) -> Self {
        Self { team_id, event_message }
    }

    pub fn get_team_id(&self) -> &str { &self.team_id }
    pub fn get_event_message(&self) -> &str { &self.event_message }
}

impl IReport for ReportTeamEvent {
    fn get_id(&self) -> ReportId { ReportId::TEAM_EVENT }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportTeamEvent {
        ReportTeamEvent::new("team1".into(), "Player banned!".into())
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::TEAM_EVENT);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "teamEvent");
    }

    #[test]
    fn fields() {
        let r = make();
        assert_eq!(r.get_team_id(), "team1");
        assert_eq!(r.get_event_message(), "Player banned!");
    }
}
