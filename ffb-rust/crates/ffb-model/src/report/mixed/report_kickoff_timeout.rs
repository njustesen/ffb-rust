use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportKickoffTimeout.java`.
#[derive(Debug, Clone)]
pub struct ReportKickoffTimeout {
    pub turn_modifier: i32,
    pub turn_number: i32,
}

impl ReportKickoffTimeout {
    pub fn new(turn_modifier: i32, turn_number: i32) -> Self {
        Self { turn_modifier, turn_number }
    }

    pub fn get_turn_modifier(&self) -> i32 { self.turn_modifier }
    pub fn get_turn_number(&self) -> i32 { self.turn_number }
}

impl IReport for ReportKickoffTimeout {
    fn get_id(&self) -> ReportId { ReportId::KICKOFF_TIMEOUT }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportKickoffTimeout {
        ReportKickoffTimeout::new(1, 4)
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::KICKOFF_TIMEOUT); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "kickoffTimeout"); }

    #[test]
    fn get_turn_number() { assert_eq!(make().get_turn_number(), 4); }
}
