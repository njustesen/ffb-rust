use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportArgueTheCallRoll.java`.
#[derive(Debug, Clone)]
pub struct ReportArgueTheCallRoll {
    pub player_id: String,
    pub successful: bool,
    pub coach_banned: bool,
    pub roll: i32,
}

impl ReportArgueTheCallRoll {
    pub fn new(player_id: String, successful: bool, coach_banned: bool, roll: i32) -> Self {
        Self { player_id, successful, coach_banned, roll }
    }

    pub fn get_player_id(&self) -> &str { &self.player_id }
    pub fn is_successful(&self) -> bool { self.successful }
    pub fn is_coach_banned(&self) -> bool { self.coach_banned }
    pub fn get_roll(&self) -> i32 { self.roll }
}

impl IReport for ReportArgueTheCallRoll {
    fn get_id(&self) -> ReportId { ReportId::ARGUE_THE_CALL }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportArgueTheCallRoll {
        ReportArgueTheCallRoll::new("p1".into(), true, false, 5)
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::ARGUE_THE_CALL);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "argueTheCall");
    }

    #[test]
    fn fields() {
        let r = make();
        assert!(r.is_successful());
        assert!(!r.is_coach_banned());
        assert_eq!(r.get_roll(), 5);
    }
}
