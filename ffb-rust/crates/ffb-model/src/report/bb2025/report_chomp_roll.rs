use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;
use crate::report::report_skill_roll::ReportSkillRoll;

/// 1:1 translation of `ReportChompRoll.java` — extends ReportSkillRoll.
#[derive(Debug, Clone)]
pub struct ReportChompRoll {
    pub base: ReportSkillRoll,
    pub chomper: String,
    pub chompee: String,
}

impl ReportChompRoll {
    pub fn new(
        player_id: Option<String>,
        successful: bool,
        roll: i32,
        minimum_roll: i32,
        re_rolled: bool,
        chomper: String,
        chompee: String,
    ) -> Self {
        Self {
            base: ReportSkillRoll::new(player_id, successful, roll, minimum_roll, re_rolled, vec![]),
            chomper,
            chompee,
        }
    }

    pub fn get_player_id(&self) -> Option<&str> { self.base.get_player_id() }
    pub fn is_successful(&self) -> bool { self.base.is_successful() }
    pub fn get_roll(&self) -> i32 { self.base.get_roll() }
    pub fn get_minimum_roll(&self) -> i32 { self.base.get_minimum_roll() }
    pub fn is_re_rolled(&self) -> bool { self.base.is_re_rolled() }
    pub fn get_chomper(&self) -> &str { &self.chomper }
    pub fn get_chompee(&self) -> &str { &self.chompee }
}

impl IReport for ReportChompRoll {
    fn get_id(&self) -> ReportId { ReportId::CHOMP_ROLL }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportChompRoll {
        ReportChompRoll::new(Some("p1".into()), true, 5, 3, false, "chomper1".into(), "chompee1".into())
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::CHOMP_ROLL);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "chompRoll");
    }

    #[test]
    fn fields() {
        let r = make();
        assert_eq!(r.get_chomper(), "chomper1");
        assert_eq!(r.get_chompee(), "chompee1");
        assert!(r.is_successful());
    }
}
