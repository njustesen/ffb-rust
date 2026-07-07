use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;
use crate::report::report_skill_roll::ReportSkillRoll;

/// 1:1 translation of `ReportDauntlessRoll.java`.
/// Extends `ReportSkillRoll`; adds strength and optional defender id.
#[derive(Debug, Clone)]
pub struct ReportDauntlessRoll {
    pub base: ReportSkillRoll,
    /// Translated from `fStrength`.
    pub strength: i32,
    /// Translated from `defenderId`.
    pub defender_id: Option<String>,
}

impl ReportDauntlessRoll {
    pub fn new(
        player_id: Option<String>,
        successful: bool,
        roll: i32,
        minimum_roll: i32,
        re_rolled: bool,
        strength: i32,
        defender_id: Option<String>,
    ) -> Self {
        Self {
            base: ReportSkillRoll::new(player_id, successful, roll, minimum_roll, re_rolled, vec![]),
            strength,
            defender_id,
        }
    }

    pub fn get_strength(&self) -> i32 {
        self.strength
    }

    pub fn get_defender_id(&self) -> Option<&str> {
        self.defender_id.as_deref()
    }
}

impl IReport for ReportDauntlessRoll {
    fn get_id(&self) -> ReportId {
        ReportId::DAUNTLESS_ROLL
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportDauntlessRoll {
        ReportDauntlessRoll::new(Some("p1".into()), true, 5, 3, false, 4, Some("d1".into()))
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::DAUNTLESS_ROLL);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "dauntlessRoll");
    }

    #[test]
    fn field_getters() {
        let r = make();
        assert_eq!(r.get_strength(), 4);
        assert_eq!(r.get_defender_id(), Some("d1"));
        assert!(r.base.is_successful());
    }

    #[test]
    fn no_defender_id() {
        let r = ReportDauntlessRoll::new(Some("p2".into()), false, 2, 4, false, 3, None);
        assert_eq!(r.get_defender_id(), None);
        assert_eq!(r.get_strength(), 3);
    }

    #[test]
    fn rerolled_dauntless() {
        let r = ReportDauntlessRoll::new(Some("p1".into()), true, 6, 3, true, 5, Some("def2".into()));
        assert!(r.base.is_re_rolled());
        assert_eq!(r.get_defender_id(), Some("def2"));
    }
}
