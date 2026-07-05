use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;
use crate::report::report_skill_roll::ReportSkillRoll;

/// 1:1 translation of `ReportBalefulHexRoll.java` — extends ReportSkillRoll.
#[derive(Debug, Clone)]
pub struct ReportBalefulHexRoll {
    pub base: ReportSkillRoll,
    pub target: Option<String>,
}

impl ReportBalefulHexRoll {
    pub fn new(
        player_id: Option<String>,
        successful: bool,
        roll: i32,
        minimum_roll: i32,
        re_rolled: bool,
        target: Option<String>,
    ) -> Self {
        Self {
            base: ReportSkillRoll::new(player_id, successful, roll, minimum_roll, re_rolled, vec![]),
            target,
        }
    }

    pub fn get_player_id(&self) -> Option<&str> { self.base.get_player_id() }
    pub fn is_successful(&self) -> bool { self.base.is_successful() }
    pub fn get_roll(&self) -> i32 { self.base.get_roll() }
    pub fn get_minimum_roll(&self) -> i32 { self.base.get_minimum_roll() }
    pub fn is_re_rolled(&self) -> bool { self.base.is_re_rolled() }
    pub fn get_target(&self) -> Option<&str> { self.target.as_deref() }
}

impl IReport for ReportBalefulHexRoll {
    fn get_id(&self) -> ReportId { ReportId::BALEFUL_HEX }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportBalefulHexRoll {
        ReportBalefulHexRoll::new(Some("p1".into()), true, 4, 2, false, Some("t1".into()))
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::BALEFUL_HEX); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "balefulHex"); }

    #[test]
    fn get_target() { assert_eq!(make().get_target(), Some("t1")); }
}
