use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;
use crate::report::report_skill_roll::ReportSkillRoll;

/// 1:1 translation of `ReportAllYouCanEatRoll.java` — extends ReportSkillRoll.
#[derive(Debug, Clone)]
pub struct ReportAllYouCanEatRoll {
    pub base: ReportSkillRoll,
}

impl ReportAllYouCanEatRoll {
    pub fn new(
        player_id: Option<String>,
        successful: bool,
        roll: i32,
        minimum_roll: i32,
        re_rolled: bool,
    ) -> Self {
        Self {
            base: ReportSkillRoll::new(player_id, successful, roll, minimum_roll, re_rolled, vec![]),
        }
    }

    pub fn get_player_id(&self) -> Option<&str> { self.base.get_player_id() }
    pub fn is_successful(&self) -> bool { self.base.is_successful() }
    pub fn get_roll(&self) -> i32 { self.base.get_roll() }
    pub fn get_minimum_roll(&self) -> i32 { self.base.get_minimum_roll() }
    pub fn is_re_rolled(&self) -> bool { self.base.is_re_rolled() }
}

impl IReport for ReportAllYouCanEatRoll {
    fn get_id(&self) -> ReportId { ReportId::ALL_YOU_CAN_EAT }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportAllYouCanEatRoll {
        ReportAllYouCanEatRoll::new(Some("p1".into()), true, 4, 2, false)
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::ALL_YOU_CAN_EAT); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "allYouCanEat"); }

    #[test]
    fn get_player_id() { assert_eq!(make().get_player_id(), Some("p1")); }
}
