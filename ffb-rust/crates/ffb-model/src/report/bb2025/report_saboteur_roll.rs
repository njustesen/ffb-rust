use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;
use crate::report::report_skill_roll::ReportSkillRoll;

/// 1:1 translation of `ReportSaboteurRoll.java` — extends ReportSkillRoll.
#[derive(Debug, Clone)]
pub struct ReportSaboteurRoll {
    pub base: ReportSkillRoll,
}

impl ReportSaboteurRoll {
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

impl IReport for ReportSaboteurRoll {
    fn get_id(&self) -> ReportId { ReportId::SABOTEUR_ROLL }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportSaboteurRoll {
        ReportSaboteurRoll::new(Some("p1".into()), true, 4, 3, false)
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::SABOTEUR_ROLL);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "saboteurRoll");
    }

    #[test]
    fn fields() {
        let r = make();
        assert_eq!(r.get_player_id(), Some("p1"));
        assert!(r.is_successful());
        assert_eq!(r.get_roll(), 4);
    }

    #[test]
    fn minimum_roll_and_rerolled() {
        let r = make();
        assert_eq!(r.get_minimum_roll(), 3);
        assert!(!r.is_re_rolled());
    }

    #[test]
    fn unsuccessful_no_player_id() {
        let r = ReportSaboteurRoll::new(None, false, 2, 4, true);
        assert!(!r.is_successful());
        assert_eq!(r.get_player_id(), None);
        assert!(r.is_re_rolled());
    }
}
