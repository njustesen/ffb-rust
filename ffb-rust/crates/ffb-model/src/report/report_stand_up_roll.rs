use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportStandUpRoll.java`.
/// Note: Java's `ReportStandUpRoll` does NOT extend `ReportSkillRoll`; it is a standalone class.
#[derive(Debug, Clone)]
pub struct ReportStandUpRoll {
    pub player_id: Option<String>,
    pub successful: bool,
    pub roll: i32,
    pub modifier: i32,
    pub re_rolled: bool,
}

impl ReportStandUpRoll {
    pub fn new(
        player_id: Option<String>,
        successful: bool,
        roll: i32,
        modifier: i32,
        re_rolled: bool,
    ) -> Self {
        Self { player_id, successful, roll, modifier, re_rolled }
    }

    pub fn get_player_id(&self) -> Option<&str> { self.player_id.as_deref() }
    pub fn is_successful(&self) -> bool { self.successful }
    pub fn get_roll(&self) -> i32 { self.roll }
    pub fn get_modifier(&self) -> i32 { self.modifier }
    pub fn get_minimum_roll(&self) -> i32 { (4 - self.modifier).max(2) }
    pub fn is_re_rolled(&self) -> bool { self.re_rolled }
}

impl IReport for ReportStandUpRoll {
    fn get_id(&self) -> ReportId { ReportId::STAND_UP_ROLL }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportStandUpRoll {
        ReportStandUpRoll::new(Some("p1".into()), true, 4, 1, false)
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::STAND_UP_ROLL);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "standUpRoll");
    }

    #[test]
    fn minimum_roll_clamped() {
        // modifier=1 → 4-1=3; modifier=3 → 4-3=1 clamped to 2
        assert_eq!(make().get_minimum_roll(), 3);
        let r = ReportStandUpRoll::new(Some("p1".into()), true, 2, 3, false);
        assert_eq!(r.get_minimum_roll(), 2);
    }
}
