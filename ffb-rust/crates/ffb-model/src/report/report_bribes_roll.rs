use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportBribesRoll.java`.
#[derive(Debug, Clone)]
pub struct ReportBribesRoll {
    pub player_id: String,
    pub successful: bool,
    pub roll: i32,
}

impl ReportBribesRoll {
    pub fn new(player_id: String, successful: bool, roll: i32) -> Self {
        Self { player_id, successful, roll }
    }

    pub fn get_player_id(&self) -> &str { &self.player_id }
    pub fn is_successful(&self) -> bool { self.successful }
    pub fn get_roll(&self) -> i32 { self.roll }
}

impl IReport for ReportBribesRoll {
    fn get_id(&self) -> ReportId { ReportId::BRIBES_ROLL }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportBribesRoll {
        ReportBribesRoll::new("p1".into(), true, 4)
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::BRIBES_ROLL);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "bribesRoll");
    }

    #[test]
    fn get_roll() {
        assert_eq!(make().get_roll(), 4);
    }

    #[test]
    fn is_successful() {
        assert!(make().is_successful());
    }

    #[test]
    fn unsuccessful_bribe() {
        let r = ReportBribesRoll::new("p2".into(), false, 2);
        assert!(!r.is_successful());
        assert_eq!(r.get_player_id(), "p2");
        assert_eq!(r.get_roll(), 2);
    }
}
