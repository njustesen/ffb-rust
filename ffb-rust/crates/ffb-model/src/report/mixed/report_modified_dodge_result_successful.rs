use crate::enums::SkillId;
use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportModifiedDodgeResultSuccessful.java`.
#[derive(Debug, Clone)]
pub struct ReportModifiedDodgeResultSuccessful {
    pub skill_id: Option<SkillId>,
}

impl ReportModifiedDodgeResultSuccessful {
    pub fn new(skill_id: Option<SkillId>) -> Self {
        Self { skill_id }
    }

    pub fn get_skill_id(&self) -> Option<SkillId> { self.skill_id }
}

impl IReport for ReportModifiedDodgeResultSuccessful {
    fn get_id(&self) -> ReportId { ReportId::MODIFIED_DODGE_RESULT_SUCCESSFUL }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportModifiedDodgeResultSuccessful {
        ReportModifiedDodgeResultSuccessful::new(None)
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::MODIFIED_DODGE_RESULT_SUCCESSFUL); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "modifiedDodgeResultSuccessful"); }

    #[test]
    fn get_skill_id_none() { assert!(make().get_skill_id().is_none()); }
}
