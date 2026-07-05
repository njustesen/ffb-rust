use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;
use crate::report::report_skill_roll::ReportSkillRoll;

/// 1:1 translation of `ReportInterceptionRoll.java`.
/// Extends `ReportSkillRoll`; adds bomb flag and ignore-agility flag.
#[derive(Debug, Clone)]
pub struct ReportInterceptionRoll {
    pub base: ReportSkillRoll,
    /// Translated from `fBomb`.
    pub bomb: bool,
    /// Translated from `ignoreAgility`.
    pub ignore_agility: bool,
}

impl ReportInterceptionRoll {
    pub fn new(
        player_id: Option<String>,
        successful: bool,
        roll: i32,
        minimum_roll: i32,
        re_rolled: bool,
        roll_modifiers: Vec<String>,
        bomb: bool,
        ignore_agility: bool,
    ) -> Self {
        Self {
            base: ReportSkillRoll::new(player_id, successful, roll, minimum_roll, re_rolled, roll_modifiers),
            bomb,
            ignore_agility,
        }
    }

    pub fn is_bomb(&self) -> bool {
        self.bomb
    }

    pub fn is_ignore_agility(&self) -> bool {
        self.ignore_agility
    }
}

impl IReport for ReportInterceptionRoll {
    fn get_id(&self) -> ReportId {
        ReportId::INTERCEPTION_ROLL
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportInterceptionRoll {
        ReportInterceptionRoll::new(
            Some("p1".into()),
            false,
            3,
            5,
            false,
            vec![],
            true,
            false,
        )
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::INTERCEPTION_ROLL);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "interceptionRoll");
    }

    #[test]
    fn field_getters() {
        let r = make();
        assert!(r.is_bomb());
        assert!(!r.is_ignore_agility());
        assert!(!r.base.is_successful());
    }
}
