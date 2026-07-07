use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;
use crate::report::report_skill_roll::ReportSkillRoll;

/// 1:1 translation of `ReportSteadyFootingRoll.java` — extends ReportSkillRoll.
#[derive(Debug, Clone)]
pub struct ReportSteadyFootingRoll {
    pub base: ReportSkillRoll,
}

impl ReportSteadyFootingRoll {
    pub fn new(player_id: Option<String>, successful: bool, roll: i32, minimum_roll: i32, re_rolled: bool) -> Self {
        Self {
            base: ReportSkillRoll::new(player_id, successful, roll, minimum_roll, re_rolled, vec![]),
        }
    }

    pub fn get_player_id(&self) -> Option<&str> { self.base.get_player_id() }
    pub fn is_successful(&self) -> bool { self.base.is_successful() }
    pub fn get_roll(&self) -> i32 { self.base.get_roll() }
    pub fn get_minimum_roll(&self) -> i32 { self.base.get_minimum_roll() }
    pub fn is_re_rolled(&self) -> bool { self.base.is_re_rolled() }
}

impl IReport for ReportSteadyFootingRoll {
    fn get_id(&self) -> ReportId { ReportId::STEADY_FOOTING_ROLL }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportSteadyFootingRoll {
        ReportSteadyFootingRoll::new(Some("p1".into()), false, 2, 4, true)
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::STEADY_FOOTING_ROLL);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "steadyFootingRoll");
    }

    #[test]
    fn fields() {
        let r = make();
        assert!(!r.is_successful());
        assert!(r.is_re_rolled());
    }

    #[test]
    fn minimum_roll_and_player_id() {
        let r = make();
        assert_eq!(r.get_minimum_roll(), 4);
        assert_eq!(r.get_player_id(), Some("p1"));
    }

    #[test]
    fn successful_not_rerolled() {
        let r = ReportSteadyFootingRoll::new(Some("p2".into()), true, 5, 3, false);
        assert!(r.is_successful());
        assert!(!r.is_re_rolled());
        assert_eq!(r.get_roll(), 5);
    }
}
