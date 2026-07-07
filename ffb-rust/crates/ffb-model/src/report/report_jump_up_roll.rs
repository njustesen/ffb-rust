use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;
use crate::report::report_skill_roll::ReportSkillRoll;

/// 1:1 translation of `ReportJumpUpRoll.java`.
/// Extends `ReportSkillRoll`; no additional fields.
#[derive(Debug, Clone)]
pub struct ReportJumpUpRoll {
    pub base: ReportSkillRoll,
}

impl ReportJumpUpRoll {
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
}

impl IReport for ReportJumpUpRoll {
    fn get_id(&self) -> ReportId {
        ReportId::JUMP_UP_ROLL
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportJumpUpRoll {
        ReportJumpUpRoll::new(Some("p1".into()), false, 1, 3, true, vec![])
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::JUMP_UP_ROLL);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "jumpUpRoll");
    }

    #[test]
    fn base_fields() {
        let r = make();
        assert!(!r.base.is_successful());
        assert!(r.base.is_re_rolled());
        assert_eq!(r.base.get_roll(), 1);
    }

    #[test]
    fn minimum_roll_and_rerolled() {
        let r = ReportJumpUpRoll::new(Some("p1".into()), true, 4, 3, true, vec![]);
        assert_eq!(r.base.get_minimum_roll(), 3);
        assert!(r.base.is_re_rolled());
    }

    #[test]
    fn unsuccessful_with_modifiers() {
        let r = ReportJumpUpRoll::new(None, false, 2, 4, false, vec!["TackleZone".into()]);
        assert!(!r.base.is_successful());
        assert_eq!(r.base.get_roll_modifiers().len(), 1);
    }
}
