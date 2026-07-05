use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportKickoffSequenceActivationsExhausted.java`.
#[derive(Debug, Clone)]
pub struct ReportKickoffSequenceActivationsExhausted {
    pub limit_reached: bool,
}

impl ReportKickoffSequenceActivationsExhausted {
    pub fn new(limit_reached: bool) -> Self {
        Self { limit_reached }
    }

    pub fn is_limit_reached(&self) -> bool { self.limit_reached }
}

impl IReport for ReportKickoffSequenceActivationsExhausted {
    fn get_id(&self) -> ReportId { ReportId::KICKOFF_SEQUENCE_ACTIVATIONS_EXHAUSTED }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportKickoffSequenceActivationsExhausted {
        ReportKickoffSequenceActivationsExhausted::new(true)
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::KICKOFF_SEQUENCE_ACTIVATIONS_EXHAUSTED); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "kickoffSequenceActivationsExhausted"); }

    #[test]
    fn is_limit_reached() { assert!(make().is_limit_reached()); }
}
