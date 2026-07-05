use crate::enums::SkillId;
use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportSkillWasted.java`.
#[derive(Debug, Clone)]
pub struct ReportSkillWasted {
    /// `fPlayerId`
    pub player_id: Option<String>,
    /// `fSkill`
    pub skill: Option<SkillId>,
}

impl ReportSkillWasted {
    pub fn new(player_id: Option<String>, skill: Option<SkillId>) -> Self {
        Self { player_id, skill }
    }

    pub fn get_player_id(&self) -> Option<&str> { self.player_id.as_deref() }
    pub fn get_skill(&self) -> Option<SkillId> { self.skill }
}

impl IReport for ReportSkillWasted {
    fn get_id(&self) -> ReportId { ReportId::SKILL_WASTED }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportSkillWasted {
        ReportSkillWasted::new(Some("p1".into()), None)
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::SKILL_WASTED); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "skillWasted"); }

    #[test]
    fn get_player_id() { assert_eq!(make().get_player_id(), Some("p1")); }
}
