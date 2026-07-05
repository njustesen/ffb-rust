use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportReferee.java`.
#[derive(Debug, Clone)]
pub struct ReportReferee {
    /// `fFoulingPlayerBanned`
    pub fouling_player_banned: bool,
    pub under_scrutiny: bool,
}

impl ReportReferee {
    pub fn new(fouling_player_banned: bool, under_scrutiny: bool) -> Self {
        Self { fouling_player_banned, under_scrutiny }
    }

    pub fn is_fouling_player_banned(&self) -> bool { self.fouling_player_banned }
    pub fn is_under_scrutiny(&self) -> bool { self.under_scrutiny }
}

impl IReport for ReportReferee {
    fn get_id(&self) -> ReportId { ReportId::REFEREE }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportReferee {
        ReportReferee::new(true, false)
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::REFEREE); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "referee"); }

    #[test]
    fn is_fouling_player_banned() { assert!(make().is_fouling_player_banned()); }

    #[test]
    fn is_under_scrutiny() { assert!(!make().is_under_scrutiny()); }
}
