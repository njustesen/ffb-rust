use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;
use crate::report::report_skill_roll::ReportSkillRoll;

/// 1:1 translation of `ReportFoulAppearanceRoll.java`.
/// Extends `ReportSkillRoll`; adds optional defender id.
#[derive(Debug, Clone)]
pub struct ReportFoulAppearanceRoll {
    pub base: ReportSkillRoll,
    /// Translated from `defenderId`.
    pub defender_id: Option<String>,
}

impl ReportFoulAppearanceRoll {
    pub fn new(
        player_id: Option<String>,
        successful: bool,
        roll: i32,
        minimum_roll: i32,
        re_rolled: bool,
        roll_modifiers: Vec<String>,
        defender_id: Option<String>,
    ) -> Self {
        Self {
            base: ReportSkillRoll::new(player_id, successful, roll, minimum_roll, re_rolled, roll_modifiers),
            defender_id,
        }
    }

    pub fn get_defender_id(&self) -> Option<&str> {
        self.defender_id.as_deref()
    }
}

impl IReport for ReportFoulAppearanceRoll {
    fn get_id(&self) -> ReportId {
        ReportId::FOUL_APPEARANCE_ROLL
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportFoulAppearanceRoll {
        ReportFoulAppearanceRoll::new(
            Some("p1".into()),
            false,
            2,
            3,
            true,
            vec![],
            Some("d1".into()),
        )
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::FOUL_APPEARANCE_ROLL);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "foulAppearanceRoll");
    }

    #[test]
    fn field_getters() {
        let r = make();
        assert_eq!(r.get_defender_id(), Some("d1"));
        assert!(!r.base.is_successful());
        assert!(r.base.is_re_rolled());
    }
}
