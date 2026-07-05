use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportReferee.java`.
#[derive(Debug, Clone)]
pub struct ReportReferee {
    pub fouling_player_banned: bool,
}

impl ReportReferee {
    pub fn new(fouling_player_banned: bool) -> Self {
        Self { fouling_player_banned }
    }

    pub fn is_fouling_player_banned(&self) -> bool { self.fouling_player_banned }
}

impl IReport for ReportReferee {
    fn get_id(&self) -> ReportId { ReportId::REFEREE }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_id() {
        assert_eq!(ReportReferee::new(true).get_id(), ReportId::REFEREE);
    }

    #[test]
    fn get_name() {
        assert_eq!(ReportReferee::new(false).get_name(), "referee");
    }

    #[test]
    fn fields() {
        assert!(ReportReferee::new(true).is_fouling_player_banned());
        assert!(!ReportReferee::new(false).is_fouling_player_banned());
    }
}
