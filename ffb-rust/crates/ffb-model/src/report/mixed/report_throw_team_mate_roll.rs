use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;
use crate::report::report_skill_roll::ReportSkillRoll;

/// 1:1 translation of `ReportThrowTeamMateRoll.java` — extends ReportSkillRoll.
#[derive(Debug, Clone)]
pub struct ReportThrowTeamMateRoll {
    pub base: ReportSkillRoll,
    /// `fThrownPlayerId`
    pub thrown_player_id: Option<String>,
    /// `fPassingDistance` — distance category name.
    pub passing_distance: Option<String>,
    /// `passResult` — PassResult name string.
    pub pass_result: Option<String>,
    pub is_kick: bool,
}

impl ReportThrowTeamMateRoll {
    pub fn new(
        thrower_id: Option<String>,
        successful: bool,
        roll: i32,
        minimum_roll: i32,
        re_rolled: bool,
        roll_modifiers: Vec<String>,
        passing_distance: Option<String>,
        thrown_player_id: Option<String>,
        pass_result: Option<String>,
        is_kick: bool,
    ) -> Self {
        Self {
            base: ReportSkillRoll::new(thrower_id, successful, roll, minimum_roll, re_rolled, roll_modifiers),
            thrown_player_id,
            passing_distance,
            pass_result,
            is_kick,
        }
    }

    pub fn get_player_id(&self) -> Option<&str> { self.base.get_player_id() }
    pub fn is_successful(&self) -> bool { self.base.is_successful() }
    pub fn get_roll(&self) -> i32 { self.base.get_roll() }
    pub fn get_minimum_roll(&self) -> i32 { self.base.get_minimum_roll() }
    pub fn is_re_rolled(&self) -> bool { self.base.is_re_rolled() }
    pub fn get_roll_modifiers(&self) -> &[String] { self.base.get_roll_modifiers() }
    pub fn get_thrown_player_id(&self) -> Option<&str> { self.thrown_player_id.as_deref() }
    pub fn get_passing_distance(&self) -> Option<&str> { self.passing_distance.as_deref() }
    pub fn get_pass_result(&self) -> Option<&str> { self.pass_result.as_deref() }
    pub fn is_kick(&self) -> bool { self.is_kick }
}

impl IReport for ReportThrowTeamMateRoll {
    fn get_id(&self) -> ReportId { ReportId::THROW_TEAM_MATE_ROLL }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportThrowTeamMateRoll {
        ReportThrowTeamMateRoll::new(
            Some("thrower".into()), true, 4, 3, false, vec![],
            Some("SHORT_PASS".into()), Some("thrown".into()), Some("ACCURATE".into()), false,
        )
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::THROW_TEAM_MATE_ROLL); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "throwTeamMateRoll"); }

    #[test]
    fn get_thrown_player_id() { assert_eq!(make().get_thrown_player_id(), Some("thrown")); }

    #[test]
    fn get_pass_result() { assert_eq!(make().get_pass_result(), Some("ACCURATE")); }

    #[test]
    fn is_kick() { assert!(!make().is_kick()); }
}
