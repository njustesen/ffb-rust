use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportKickoffSequenceActivationsCount.java`.
#[derive(Debug, Clone)]
pub struct ReportKickoffSequenceActivationsCount {
    pub amount: i32,
    pub available: i32,
    pub limit: i32,
}

impl ReportKickoffSequenceActivationsCount {
    pub fn new(amount: i32, available: i32, limit: i32) -> Self {
        Self { amount, available, limit }
    }

    pub fn get_amount(&self) -> i32 { self.amount }
    pub fn get_available(&self) -> i32 { self.available }
    pub fn get_limit(&self) -> i32 { self.limit }
}

impl IReport for ReportKickoffSequenceActivationsCount {
    fn get_id(&self) -> ReportId { ReportId::KICKOFF_SEQUENCE_ACTIVATIONS_COUNT }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportKickoffSequenceActivationsCount {
        ReportKickoffSequenceActivationsCount::new(2, 5, 3)
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::KICKOFF_SEQUENCE_ACTIVATIONS_COUNT); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "kickoffSequenceActivationsCount"); }

    #[test]
    fn get_limit() { assert_eq!(make().get_limit(), 3); }
}
