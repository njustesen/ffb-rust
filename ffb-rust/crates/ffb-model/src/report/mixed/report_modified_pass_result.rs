use crate::enums::SkillId;
use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportModifiedPassResult.java`.
#[derive(Debug, Clone)]
pub struct ReportModifiedPassResult {
    pub skill_id: Option<SkillId>,
    pub pass_result: String,
}

impl ReportModifiedPassResult {
    pub fn new(skill_id: Option<SkillId>, pass_result: String) -> Self {
        Self { skill_id, pass_result }
    }

    pub fn get_skill_id(&self) -> Option<SkillId> { self.skill_id }
    pub fn get_pass_result(&self) -> &str { &self.pass_result }
}

impl IReport for ReportModifiedPassResult {
    fn get_id(&self) -> ReportId { ReportId::MODIFIED_PASS_RESULT }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportModifiedPassResult {
        ReportModifiedPassResult::new(None, "ACCURATE".into())
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::MODIFIED_PASS_RESULT); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "modifiedPassResult"); }

    #[test]
    fn get_pass_result() { assert_eq!(make().get_pass_result(), "ACCURATE"); }
}
