use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportReceiveChoice.java`.
#[derive(Debug, Clone)]
pub struct ReportReceiveChoice {
    pub team_id: String,
    pub receive_choice: bool,
}

impl ReportReceiveChoice {
    pub fn new(team_id: String, receive_choice: bool) -> Self {
        Self { team_id, receive_choice }
    }

    pub fn get_team_id(&self) -> &str { &self.team_id }
    pub fn is_receive_choice(&self) -> bool { self.receive_choice }
}

impl IReport for ReportReceiveChoice {
    fn get_id(&self) -> ReportId { ReportId::RECEIVE_CHOICE }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportReceiveChoice {
        ReportReceiveChoice::new("team1".into(), true)
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::RECEIVE_CHOICE);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "receiveChoice");
    }

    #[test]
    fn fields() {
        let r = make();
        assert_eq!(r.get_team_id(), "team1");
        assert!(r.is_receive_choice());
    }
}
