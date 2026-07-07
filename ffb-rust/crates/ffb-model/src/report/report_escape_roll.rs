use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;
use crate::report::report_skill_roll::ReportSkillRoll;

/// 1:1 translation of `ReportEscapeRoll.java`.
/// Extends `ReportSkillRoll`; no additional fields.
#[derive(Debug, Clone)]
pub struct ReportEscapeRoll {
    pub base: ReportSkillRoll,
}

impl ReportEscapeRoll {
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

impl IReport for ReportEscapeRoll {
    fn get_id(&self) -> ReportId {
        ReportId::ESCAPE_ROLL
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportEscapeRoll {
        ReportEscapeRoll::new(Some("p1".into()), true, 3, 2, false, vec![])
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::ESCAPE_ROLL);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "escapeRoll");
    }

    #[test]
    fn base_fields() {
        let r = make();
        assert_eq!(r.base.get_player_id(), Some("p1"));
        assert!(r.base.is_successful());
        assert_eq!(r.base.get_roll(), 3);
    }

    #[test]
    fn minimum_roll_and_rerolled() {
        let r = ReportEscapeRoll::new(Some("p1".into()), true, 4, 3, true, vec![]);
        assert_eq!(r.base.get_minimum_roll(), 3);
        assert!(r.base.is_re_rolled());
    }

    #[test]
    fn unsuccessful_with_modifiers() {
        let r = ReportEscapeRoll::new(None, false, 2, 4, false, vec!["TackleZone".into()]);
        assert!(!r.base.is_successful());
        assert_eq!(r.base.get_roll_modifiers().len(), 1);
    }
}
