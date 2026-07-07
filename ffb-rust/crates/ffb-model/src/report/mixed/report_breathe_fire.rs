use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;
use crate::report::report_skill_roll::ReportSkillRoll;

/// 1:1 translation of `ReportBreatheFire.java` — extends ReportSkillRoll.
#[derive(Debug, Clone)]
pub struct ReportBreatheFire {
    pub base: ReportSkillRoll,
    pub defender_id: Option<String>,
    pub strong_opponent: bool,
    pub result: String,
}

impl ReportBreatheFire {
    pub fn new(
        player_id: Option<String>,
        successful: bool,
        roll: i32,
        minimum_roll: i32,
        re_rolled: bool,
        defender_id: Option<String>,
        strong_opponent: bool,
        result: String,
    ) -> Self {
        Self {
            base: ReportSkillRoll::new(player_id, successful, roll, minimum_roll, re_rolled, vec![]),
            defender_id,
            strong_opponent,
            result,
        }
    }

    pub fn get_player_id(&self) -> Option<&str> { self.base.get_player_id() }
    pub fn is_successful(&self) -> bool { self.base.is_successful() }
    pub fn get_roll(&self) -> i32 { self.base.get_roll() }
    pub fn get_minimum_roll(&self) -> i32 { self.base.get_minimum_roll() }
    pub fn is_re_rolled(&self) -> bool { self.base.is_re_rolled() }
    pub fn get_defender_id(&self) -> Option<&str> { self.defender_id.as_deref() }
    pub fn is_strong_opponent(&self) -> bool { self.strong_opponent }
    pub fn get_result(&self) -> &str { &self.result }
}

impl IReport for ReportBreatheFire {
    fn get_id(&self) -> ReportId { ReportId::BREATHE_FIRE }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportBreatheFire {
        ReportBreatheFire::new(
            Some("p1".into()), true, 4, 2, false,
            Some("d1".into()), false, "HIT".into(),
        )
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::BREATHE_FIRE); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "breatheFire"); }

    #[test]
    fn get_result() { assert_eq!(make().get_result(), "HIT"); }

    #[test]
    fn get_defender_id_and_successful() {
        assert_eq!(make().get_defender_id(), Some("d1"));
        assert!(make().is_successful());
    }

    #[test]
    fn strong_opponent_false() { assert!(!make().is_strong_opponent()); }
}
