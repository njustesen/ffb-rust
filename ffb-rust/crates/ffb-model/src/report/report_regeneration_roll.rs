use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;
use crate::report::report_skill_roll::ReportSkillRoll;

/// 1:1 translation of `ReportRegenerationRoll.java` — extends ReportSkillRoll.
#[derive(Debug, Clone)]
pub struct ReportRegenerationRoll {
    pub base: ReportSkillRoll,
}

impl ReportRegenerationRoll {
    pub fn new(
        player_id: Option<String>,
        successful: bool,
        roll: i32,
        minimum_roll: i32,
        re_rolled: bool,
        roll_modifiers: Vec<String>,
    ) -> Self {
        Self {
            base: ReportSkillRoll::new(player_id, successful, roll, minimum_roll, re_rolled, roll_modifiers),
        }
    }

    pub fn get_player_id(&self) -> Option<&str> { self.base.get_player_id() }
    pub fn is_successful(&self) -> bool { self.base.is_successful() }
    pub fn get_roll(&self) -> i32 { self.base.get_roll() }
    pub fn get_minimum_roll(&self) -> i32 { self.base.get_minimum_roll() }
    pub fn is_re_rolled(&self) -> bool { self.base.is_re_rolled() }
    pub fn get_roll_modifiers(&self) -> &[String] { self.base.get_roll_modifiers() }
}

impl IReport for ReportRegenerationRoll {
    fn get_id(&self) -> ReportId { ReportId::REGENERATION_ROLL }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportRegenerationRoll {
        ReportRegenerationRoll::new(Some("p1".into()), true, 4, 4, false, vec![])
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::REGENERATION_ROLL);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "regenerationRoll");
    }

    #[test]
    fn get_player_id() {
        assert_eq!(make().get_player_id(), Some("p1"));
    }

    #[test]
    fn minimum_roll_and_rerolled() {
        let r = ReportRegenerationRoll::new(Some("p1".into()), true, 4, 3, true, vec![]);
        assert_eq!(r.get_minimum_roll(), 3);
        assert!(r.is_re_rolled());
    }

    #[test]
    fn unsuccessful_with_modifiers() {
        let r = ReportRegenerationRoll::new(None, false, 2, 4, false, vec!["TackleZone".into()]);
        assert!(!r.is_successful());
        assert_eq!(r.get_roll_modifiers().len(), 1);
    }
}
