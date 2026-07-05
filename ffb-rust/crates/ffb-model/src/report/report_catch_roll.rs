use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;
use crate::report::report_skill_roll::ReportSkillRoll;

/// 1:1 translation of `ReportCatchRoll.java` — extends ReportSkillRoll.
#[derive(Debug, Clone)]
pub struct ReportCatchRoll {
    pub base: ReportSkillRoll,
    pub bomb: bool,
}

impl ReportCatchRoll {
    pub fn new(
        player_id: Option<String>,
        successful: bool,
        roll: i32,
        minimum_roll: i32,
        re_rolled: bool,
        roll_modifiers: Vec<String>,
        bomb: bool,
    ) -> Self {
        Self {
            base: ReportSkillRoll::new(player_id, successful, roll, minimum_roll, re_rolled, roll_modifiers),
            bomb,
        }
    }

    pub fn get_player_id(&self) -> Option<&str> { self.base.get_player_id() }
    pub fn is_successful(&self) -> bool { self.base.is_successful() }
    pub fn get_roll(&self) -> i32 { self.base.get_roll() }
    pub fn get_minimum_roll(&self) -> i32 { self.base.get_minimum_roll() }
    pub fn is_re_rolled(&self) -> bool { self.base.is_re_rolled() }
    pub fn get_roll_modifiers(&self) -> &[String] { self.base.get_roll_modifiers() }
    pub fn is_bomb(&self) -> bool { self.bomb }
}

impl IReport for ReportCatchRoll {
    fn get_id(&self) -> ReportId { ReportId::CATCH_ROLL }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportCatchRoll {
        ReportCatchRoll::new(Some("p1".into()), true, 4, 3, false, vec![], false)
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::CATCH_ROLL);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "catchRoll");
    }

    #[test]
    fn is_bomb() {
        assert!(!make().is_bomb());
        let bomb = ReportCatchRoll::new(Some("p1".into()), false, 2, 4, false, vec![], true);
        assert!(bomb.is_bomb());
    }
}
