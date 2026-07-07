use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;
use crate::report::report_skill_roll::ReportSkillRoll;

/// 1:1 translation of `ReportPassRoll.java` — extends ReportSkillRoll.
#[derive(Debug, Clone)]
pub struct ReportPassRoll {
    pub base: ReportSkillRoll,
    pub passing_distance: Option<String>,
    pub hail_mary_pass: bool,
    pub bomb: bool,
    pub result: String,
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
        hail_mary_pass: bool,
        bomb: bool,
        result: String,
    ) -> Self {
        Self {
            base: ReportSkillRoll::new(player_id, successful, roll, minimum_roll, re_rolled, roll_modifiers),
            passing_distance,
            hail_mary_pass,
            bomb,
            result,
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
    pub fn get_result(&self) -> &str { &self.result }
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
            Some("LONG_PASS".into()), false, false, "ACCURATE".into()
        )
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::PASS_ROLL);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "passRoll");
    }

    #[test]
    fn fields() {
        let r = make();
        assert_eq!(r.get_passing_distance(), Some("LONG_PASS"));
        assert!(!r.is_bomb());
        assert_eq!(r.get_result(), "ACCURATE");
    }

    #[test]
    fn minimum_roll_and_rerolled() {
        let r = make();
        assert_eq!(r.get_minimum_roll(), 3);
        assert!(!r.is_re_rolled());
    }

    #[test]
    fn bomb_and_hail_mary_flags() {
        let r = ReportPassRoll::new(
            None, false, 1, 4, true, vec!["modifier".into()],
            None, true, true, "INACCURATE".into(),
        );
        assert!(r.is_bomb());
        assert!(r.is_hail_mary_pass());
        assert!(r.is_re_rolled());
        assert_eq!(r.get_roll_modifiers().len(), 1);
        assert_eq!(r.get_passing_distance(), None);
    }
}
