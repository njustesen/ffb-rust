use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportKickTeamMateFumble.java`.
#[derive(Debug, Clone, Default)]
pub struct ReportKickTeamMateFumble;

impl ReportKickTeamMateFumble {
    pub fn new() -> Self {
        Self
    }
}

impl IReport for ReportKickTeamMateFumble {
    fn get_id(&self) -> ReportId { ReportId::KICK_TEAM_MATE_FUMBLE }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportKickTeamMateFumble {
        ReportKickTeamMateFumble::new()
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::KICK_TEAM_MATE_FUMBLE); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "kickTeamMateFumble"); }

    #[test]
    fn is_default() {
        let r = ReportKickTeamMateFumble::default();
        assert_eq!(r.get_id(), ReportId::KICK_TEAM_MATE_FUMBLE);
    }
}
