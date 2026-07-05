use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;
use crate::report::report_skill_roll::ReportSkillRoll;

/// 1:1 translation of `ReportChainsawRoll.java` — extends ReportSkillRoll.
#[derive(Debug, Clone)]
pub struct ReportChainsawRoll {
    pub base: ReportSkillRoll,
    /// Nullable in Java — `None` when not set.
    pub defender_id: Option<String>,
}

impl ReportChainsawRoll {
    pub fn new(
        player_id: Option<String>,
        successful: bool,
        roll: i32,
        minimum_roll: i32,
        re_rolled: bool,
        roll_modifiers: Vec<String>,
        defender_id: Option<String>,
    ) -> Self {
        Self {
            base: ReportSkillRoll::new(player_id, successful, roll, minimum_roll, re_rolled, roll_modifiers),
            defender_id,
        }
    }

    pub fn get_player_id(&self) -> Option<&str> { self.base.get_player_id() }
    pub fn is_successful(&self) -> bool { self.base.is_successful() }
    pub fn get_roll(&self) -> i32 { self.base.get_roll() }
    pub fn get_minimum_roll(&self) -> i32 { self.base.get_minimum_roll() }
    pub fn is_re_rolled(&self) -> bool { self.base.is_re_rolled() }
    pub fn get_roll_modifiers(&self) -> &[String] { self.base.get_roll_modifiers() }
    pub fn get_defender_id(&self) -> Option<&str> { self.defender_id.as_deref() }
}

impl IReport for ReportChainsawRoll {
    fn get_id(&self) -> ReportId { ReportId::CHAINSAW_ROLL }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportChainsawRoll {
        ReportChainsawRoll::new(Some("p1".into()), true, 5, 2, false, vec![], Some("def1".into()))
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::CHAINSAW_ROLL);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "chainsawRoll");
    }

    #[test]
    fn get_defender_id() {
        assert_eq!(make().get_defender_id(), Some("def1"));
    }
}
