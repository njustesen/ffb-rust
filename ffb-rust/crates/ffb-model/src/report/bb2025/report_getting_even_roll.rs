use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;
use crate::report::report_skill_roll::ReportSkillRoll;

/// 1:1 translation of `ReportGettingEvenRoll.java` — extends ReportSkillRoll.
#[derive(Debug, Clone)]
pub struct ReportGettingEvenRoll {
    pub base: ReportSkillRoll,
    pub keyword: String,
}

impl ReportGettingEvenRoll {
    pub fn new(
        player_id: Option<String>,
        successful: bool,
        roll: i32,
        minimum_roll: i32,
        re_rolled: bool,
        keyword: String,
    ) -> Self {
        Self {
            base: ReportSkillRoll::new(player_id, successful, roll, minimum_roll, re_rolled, vec![]),
            keyword,
        }
    }

    pub fn get_player_id(&self) -> Option<&str> { self.base.get_player_id() }
    pub fn is_successful(&self) -> bool { self.base.is_successful() }
    pub fn get_roll(&self) -> i32 { self.base.get_roll() }
    pub fn get_minimum_roll(&self) -> i32 { self.base.get_minimum_roll() }
    pub fn is_re_rolled(&self) -> bool { self.base.is_re_rolled() }
    pub fn get_keyword(&self) -> &str { &self.keyword }
}

impl IReport for ReportGettingEvenRoll {
    fn get_id(&self) -> ReportId { ReportId::GETTING_EVEN_ROLL }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportGettingEvenRoll {
        ReportGettingEvenRoll::new(Some("p1".into()), true, 4, 3, false, "Agility".into())
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::GETTING_EVEN_ROLL);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "gettingEvenRoll");
    }

    #[test]
    fn fields() {
        let r = make();
        assert_eq!(r.get_keyword(), "Agility");
        assert!(r.is_successful());
    }

    #[test]
    fn minimum_roll_and_rerolled() {
        let r = make();
        assert_eq!(r.get_minimum_roll(), 3);
        assert!(!r.is_re_rolled());
    }

    #[test]
    fn unsuccessful_with_player_id() {
        let r = ReportGettingEvenRoll::new(Some("p2".into()), false, 2, 5, true, "Strength".into());
        assert!(!r.is_successful());
        assert_eq!(r.get_player_id(), Some("p2"));
        assert_eq!(r.get_keyword(), "Strength");
    }
}
