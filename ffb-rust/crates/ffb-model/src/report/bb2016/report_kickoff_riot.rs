use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportKickoffRiot.java`.
#[derive(Debug, Clone)]
pub struct ReportKickoffRiot {
    pub roll: i32,
    pub turn_modifier: i32,
}

impl ReportKickoffRiot {
    pub fn new(roll: i32, turn_modifier: i32) -> Self {
        Self { roll, turn_modifier }
    }

    pub fn get_roll(&self) -> i32 { self.roll }
    pub fn get_turn_modifier(&self) -> i32 { self.turn_modifier }
}

impl IReport for ReportKickoffRiot {
    fn get_id(&self) -> ReportId { ReportId::KICKOFF_RIOT }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportKickoffRiot {
        ReportKickoffRiot::new(3, -1)
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::KICKOFF_RIOT);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "kickoffRiot");
    }

    #[test]
    fn fields() {
        let r = make();
        assert_eq!(r.get_roll(), 3);
        assert_eq!(r.get_turn_modifier(), -1);
    }
}
