use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;
use crate::report::report_skill_roll::ReportSkillRoll;

/// 1:1 translation of `ReportConfusionRoll.java`.
/// Extends `ReportSkillRoll`; adds the confusion skill name.
#[derive(Debug, Clone)]
pub struct ReportConfusionRoll {
    pub base: ReportSkillRoll,
    /// Translated from `fConfusionSkill` (Skill → SkillId name as String).
    pub confusion_skill: Option<String>,
}

impl ReportConfusionRoll {
    pub fn new(
        player_id: Option<String>,
        successful: bool,
        roll: i32,
        minimum_roll: i32,
        re_rolled: bool,
        confusion_skill: Option<String>,
    ) -> Self {
        Self {
            base: ReportSkillRoll::new(player_id, successful, roll, minimum_roll, re_rolled, vec![]),
            confusion_skill,
        }
    }

    pub fn get_confusion_skill(&self) -> Option<&str> {
        self.confusion_skill.as_deref()
    }
}

impl IReport for ReportConfusionRoll {
    fn get_id(&self) -> ReportId {
        ReportId::CONFUSION_ROLL
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportConfusionRoll {
        ReportConfusionRoll::new(Some("p1".into()), true, 4, 2, false, Some("Confusion".into()))
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::CONFUSION_ROLL);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "confusionRoll");
    }

    #[test]
    fn confusion_skill_getter() {
        let r = make();
        assert_eq!(r.get_confusion_skill(), Some("Confusion"));
        assert!(r.base.is_successful());
        assert_eq!(r.base.get_roll(), 4);
    }

    #[test]
    fn no_confusion_skill() {
        let r = ReportConfusionRoll::new(Some("p2".into()), false, 1, 3, false, None);
        assert_eq!(r.get_confusion_skill(), None);
        assert!(!r.base.is_successful());
    }

    #[test]
    fn rerolled_confusion() {
        let r = ReportConfusionRoll::new(Some("p1".into()), true, 5, 2, true, Some("Bone Head".into()));
        assert!(r.base.is_re_rolled());
        assert_eq!(r.get_confusion_skill(), Some("Bone Head"));
    }
}
