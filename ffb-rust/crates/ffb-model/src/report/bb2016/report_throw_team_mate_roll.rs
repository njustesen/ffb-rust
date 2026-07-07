use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;
use crate::report::report_skill_roll::ReportSkillRoll;

/// 1:1 translation of `ReportThrowTeamMateRoll.java` — extends ReportSkillRoll.
#[derive(Debug, Clone)]
pub struct ReportThrowTeamMateRoll {
    pub base: ReportSkillRoll,
    pub thrown_player_id: String,
    pub passing_distance: Option<String>,
}

impl ReportThrowTeamMateRoll {
    pub fn new(
        player_id: Option<String>,
        successful: bool,
        roll: i32,
        minimum_roll: i32,
        re_rolled: bool,
        roll_modifiers: Vec<String>,
        passing_distance: Option<String>,
        thrown_player_id: String,
    ) -> Self {
        Self {
            base: ReportSkillRoll::new(player_id, successful, roll, minimum_roll, re_rolled, roll_modifiers),
            thrown_player_id,
            passing_distance,
        }
    }

    pub fn get_player_id(&self) -> Option<&str> { self.base.get_player_id() }
    pub fn is_successful(&self) -> bool { self.base.is_successful() }
    pub fn get_roll(&self) -> i32 { self.base.get_roll() }
    pub fn get_minimum_roll(&self) -> i32 { self.base.get_minimum_roll() }
    pub fn is_re_rolled(&self) -> bool { self.base.is_re_rolled() }
    pub fn get_roll_modifiers(&self) -> &[String] { self.base.get_roll_modifiers() }
    pub fn get_thrown_player_id(&self) -> &str { &self.thrown_player_id }
    pub fn get_passing_distance(&self) -> Option<&str> { self.passing_distance.as_deref() }
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
            Some("SHORT_PASS".into()), "thrown".into()
        )
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::THROW_TEAM_MATE_ROLL);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "throwTeamMateRoll");
    }

    #[test]
    fn fields() {
        let r = make();
        assert_eq!(r.get_thrown_player_id(), "thrown");
        assert_eq!(r.get_passing_distance(), Some("SHORT_PASS"));
    }

    #[test]
    fn minimum_roll_and_rerolled() {
        let r = make();
        assert_eq!(r.get_minimum_roll(), 3);
        assert!(!r.is_re_rolled());
    }

    #[test]
    fn unsuccessful_with_modifiers() {
        let r = ReportThrowTeamMateRoll::new(
            None, false, 1, 4, true, vec!["Strong Arm".into()],
            None, "victim".into(),
        );
        assert!(!r.is_successful());
        assert!(r.is_re_rolled());
        assert_eq!(r.get_roll_modifiers().len(), 1);
        assert_eq!(r.get_passing_distance(), None);
    }
}
