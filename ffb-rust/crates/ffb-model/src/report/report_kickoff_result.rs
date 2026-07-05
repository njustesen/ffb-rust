use crate::enums::KickoffResult;
use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportKickoffResult.java`.
#[derive(Debug, Clone)]
pub struct ReportKickoffResult {
    /// Translated from `fKickoffResult`.
    pub kickoff_result: KickoffResult,
    /// Translated from `fKickoffRoll` (`int[]`).
    pub kickoff_roll: Vec<i32>,
}

impl ReportKickoffResult {
    pub fn new(kickoff_result: KickoffResult, kickoff_roll: Vec<i32>) -> Self {
        Self { kickoff_result, kickoff_roll }
    }

    pub fn get_kickoff_result(&self) -> &KickoffResult {
        &self.kickoff_result
    }

    pub fn get_kickoff_roll(&self) -> &[i32] {
        &self.kickoff_roll
    }
}

impl IReport for ReportKickoffResult {
    fn get_id(&self) -> ReportId {
        ReportId::KICKOFF_RESULT
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportKickoffResult {
        ReportKickoffResult::new(KickoffResult::Blitz, vec![3, 4])
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::KICKOFF_RESULT);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "kickoffResult");
    }

    #[test]
    fn field_getters() {
        let r = make();
        assert_eq!(r.get_kickoff_roll(), &[3, 4]);
        assert_eq!(r.get_kickoff_result(), &KickoffResult::Blitz);
    }
}
