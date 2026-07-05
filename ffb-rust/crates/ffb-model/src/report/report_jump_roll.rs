use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;
use crate::report::report_skill_roll::ReportSkillRoll;

/// 1:1 translation of `ReportJumpRoll.java`.
/// Extends `ReportSkillRoll`; no additional fields.
/// Note: `ReportId::JUMP_ROLL` maps to name `"leapRoll"` (see Java source).
#[derive(Debug, Clone)]
pub struct ReportJumpRoll {
    pub base: ReportSkillRoll,
}

impl ReportJumpRoll {
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

impl IReport for ReportJumpRoll {
    fn get_id(&self) -> ReportId {
        ReportId::JUMP_ROLL
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportJumpRoll {
        ReportJumpRoll::new(Some("p1".into()), true, 4, 3, false, vec![])
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::JUMP_ROLL);
    }

    #[test]
    fn get_name() {
        // JUMP_ROLL maps to "leapRoll" in the Java source
        assert_eq!(make().get_name(), "leapRoll");
    }

    #[test]
    fn base_fields() {
        let r = make();
        assert!(r.base.is_successful());
        assert_eq!(r.base.get_roll(), 4);
        assert_eq!(r.base.get_minimum_roll(), 3);
    }
}
