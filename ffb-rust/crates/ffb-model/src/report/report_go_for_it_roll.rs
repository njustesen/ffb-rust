use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;
use crate::report::report_skill_roll::ReportSkillRoll;

/// 1:1 translation of `ReportGoForItRoll.java`.
/// Extends `ReportSkillRoll`; no additional fields.
#[derive(Debug, Clone)]
pub struct ReportGoForItRoll {
    pub base: ReportSkillRoll,
}

impl ReportGoForItRoll {
    pub fn new(
        player_id: Option<String>,
        successful: bool,
        roll: i32,
        minimum_roll: i32,
        re_rolled: bool,
        roll_modifiers: Vec<String>,
    ) -> Self {
        Self {
            base: ReportSkillRoll::new(player_id, successful, roll, minimum_roll, re_rolled, roll_modifiers),
        }
    }
}

impl IReport for ReportGoForItRoll {
    fn get_id(&self) -> ReportId {
        ReportId::GO_FOR_IT_ROLL
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportGoForItRoll {
        ReportGoForItRoll::new(Some("p1".into()), true, 2, 2, false, vec![])
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::GO_FOR_IT_ROLL);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "goForItRoll");
    }

    #[test]
    fn base_fields() {
        let r = make();
        assert_eq!(r.base.get_minimum_roll(), 2);
        assert!(r.base.is_successful());
        assert!(!r.base.is_re_rolled());
    }
}
