use crate::enums::SkillId;
use crate::model::skill_use::SkillUse;
use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportSkillUse.java`.
/// `Skill` (Java) is represented as `SkillId`.
#[derive(Debug, Clone)]
pub struct ReportSkillUse {
    pub player_id: Option<String>,
    pub skill: SkillId,
    pub used: bool,
    pub skill_use: SkillUse,
}

impl ReportSkillUse {
    pub fn new(
        player_id: Option<String>,
        skill: SkillId,
        used: bool,
        skill_use: SkillUse,
    ) -> Self {
        Self { player_id, skill, used, skill_use }
    }

    pub fn get_player_id(&self) -> Option<&str> { self.player_id.as_deref() }
    pub fn get_skill(&self) -> SkillId { self.skill }
    pub fn is_used(&self) -> bool { self.used }
    pub fn get_skill_use(&self) -> SkillUse { self.skill_use }
}

impl IReport for ReportSkillUse {
    fn get_id(&self) -> ReportId { ReportId::SKILL_USE }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportSkillUse {
        ReportSkillUse::new(Some("p1".into()), SkillId::Block, true, SkillUse::BRING_DOWN_OPPONENT)
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::SKILL_USE);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "skillUse");
    }

    #[test]
    fn fields() {
        let r = make();
        assert_eq!(r.get_player_id(), Some("p1"));
        assert_eq!(r.get_skill(), SkillId::Block);
        assert!(r.is_used());
        assert_eq!(r.get_skill_use(), SkillUse::BRING_DOWN_OPPONENT);
    }

    #[test]
    fn not_used() {
        let r = ReportSkillUse::new(Some("p2".into()), SkillId::Block, false, SkillUse::WOULD_NOT_HELP);
        assert!(!r.is_used());
        assert_eq!(r.get_skill_use(), SkillUse::WOULD_NOT_HELP);
    }

    #[test]
    fn no_player_id() {
        let r = ReportSkillUse::new(None, SkillId::Block, true, SkillUse::STOP_OPPONENT);
        assert_eq!(r.get_player_id(), None);
        assert_eq!(r.get_skill_use(), SkillUse::STOP_OPPONENT);
    }
}
