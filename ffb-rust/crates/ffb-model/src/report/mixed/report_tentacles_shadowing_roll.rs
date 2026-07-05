use crate::enums::SkillId;
use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportTentaclesShadowingRoll.java`.
#[derive(Debug, Clone)]
pub struct ReportTentaclesShadowingRoll {
    /// `fSkill`
    pub skill: Option<SkillId>,
    /// `fDefenderId`
    pub defender_id: Option<String>,
    /// `fRoll`
    pub roll: i32,
    /// `fSuccessful`
    pub successful: bool,
    /// `fMinimumRoll`
    pub minimum_roll: i32,
    /// `fReRolled`
    pub re_rolled: bool,
}

impl ReportTentaclesShadowingRoll {
    pub fn new(
        skill: Option<SkillId>,
        defender_id: Option<String>,
        roll: i32,
        successful: bool,
        minimum_roll: i32,
        re_rolled: bool,
    ) -> Self {
        Self { skill, defender_id, roll, successful, minimum_roll, re_rolled }
    }

    pub fn get_skill(&self) -> Option<SkillId> { self.skill }
    pub fn get_defender_id(&self) -> Option<&str> { self.defender_id.as_deref() }
    pub fn get_roll(&self) -> i32 { self.roll }
    pub fn is_successful(&self) -> bool { self.successful }
    pub fn get_minimum_roll(&self) -> i32 { self.minimum_roll }
    pub fn is_re_rolled(&self) -> bool { self.re_rolled }
}

impl IReport for ReportTentaclesShadowingRoll {
    fn get_id(&self) -> ReportId { ReportId::TENTACLES_SHADOWING_ROLL }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportTentaclesShadowingRoll {
        ReportTentaclesShadowingRoll::new(None, Some("d1".into()), 4, true, 3, false)
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::TENTACLES_SHADOWING_ROLL); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "tentaclesShadowingRoll"); }

    #[test]
    fn get_defender_id() { assert_eq!(make().get_defender_id(), Some("d1")); }

    #[test]
    fn is_successful() { assert!(make().is_successful()); }
}
