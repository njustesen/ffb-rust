use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;
use crate::report::report_skill_roll::ReportSkillRoll;

/// 1:1 translation of `ReportPassRoll.java` — extends ReportSkillRoll.
#[derive(Debug, Clone)]
pub struct ReportPassRoll {
    pub base: ReportSkillRoll,
    /// `fPassingDistance` — distance category name (e.g. "SHORT_PASS").
    pub passing_distance: Option<String>,
    /// `fHailMaryPass`
    pub hail_mary_pass: bool,
    /// `fBomb`
    pub bomb: bool,
    /// `result` — PassResult name string.
    pub result: Option<String>,
    /// `statBasedRollModifier` — modifier name, if any.
    pub stat_based_roll_modifier: Option<String>,
}

impl ReportPassRoll {
    pub fn new(
        player_id: Option<String>,
        successful: bool,
        roll: i32,
        minimum_roll: i32,
        re_rolled: bool,
        roll_modifiers: Vec<String>,
        passing_distance: Option<String>,
        bomb: bool,
        result: Option<String>,
        hail_mary_pass: bool,
        stat_based_roll_modifier: Option<String>,
    ) -> Self {
        Self {
            base: ReportSkillRoll::new(player_id, successful, roll, minimum_roll, re_rolled, roll_modifiers),
            passing_distance,
            hail_mary_pass,
            bomb,
            result,
            stat_based_roll_modifier,
        }
    }

    pub fn get_player_id(&self) -> Option<&str> { self.base.get_player_id() }
    pub fn is_successful(&self) -> bool { self.base.is_successful() }
    pub fn get_roll(&self) -> i32 { self.base.get_roll() }
    pub fn get_minimum_roll(&self) -> i32 { self.base.get_minimum_roll() }
    pub fn is_re_rolled(&self) -> bool { self.base.is_re_rolled() }
    pub fn get_roll_modifiers(&self) -> &[String] { self.base.get_roll_modifiers() }
    pub fn get_passing_distance(&self) -> Option<&str> { self.passing_distance.as_deref() }
    pub fn is_hail_mary_pass(&self) -> bool { self.hail_mary_pass }
    pub fn is_bomb(&self) -> bool { self.bomb }
    pub fn get_result(&self) -> Option<&str> { self.result.as_deref() }
    pub fn get_stat_based_roll_modifier(&self) -> Option<&str> { self.stat_based_roll_modifier.as_deref() }
}

impl IReport for ReportPassRoll {
    fn get_id(&self) -> ReportId { ReportId::PASS_ROLL }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportPassRoll {
        ReportPassRoll::new(
            Some("p1".into()), true, 4, 3, false, vec![],
            Some("SHORT_PASS".into()), false, Some("ACCURATE".into()), false, None,
        )
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::PASS_ROLL); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "passRoll"); }

    #[test]
    fn get_result() { assert_eq!(make().get_result(), Some("ACCURATE")); }

    #[test]
    fn get_passing_distance() { assert_eq!(make().get_passing_distance(), Some("SHORT_PASS")); }
    #[test]
    fn get_name_is_nonempty() {
        assert!(!make().get_name().is_empty());
    }
}
