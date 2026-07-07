use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportOfficiousRefRoll.java`.
#[derive(Debug, Clone)]
pub struct ReportOfficiousRefRoll {
    pub roll: i32,
    pub player_id: String,
}

impl ReportOfficiousRefRoll {
    pub fn new(roll: i32, player_id: String) -> Self {
        Self { roll, player_id }
    }

    pub fn get_roll(&self) -> i32 { self.roll }
    pub fn get_player_id(&self) -> &str { &self.player_id }
}

impl IReport for ReportOfficiousRefRoll {
    fn get_id(&self) -> ReportId { ReportId::OFFICIOUS_REF_ROLL }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportOfficiousRefRoll {
        ReportOfficiousRefRoll::new(4, "p1".into())
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::OFFICIOUS_REF_ROLL);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "officiousRefRoll");
    }

    #[test]
    fn fields() {
        let r = make();
        assert_eq!(r.get_roll(), 4);
        assert_eq!(r.get_player_id(), "p1");
    }

    #[test]
    fn different_roll_value() {
        let r = ReportOfficiousRefRoll::new(1, "p2".into());
        assert_eq!(r.get_roll(), 1);
        assert_eq!(r.get_player_id(), "p2");
    }

    #[test]
    fn max_roll_value() {
        let r = ReportOfficiousRefRoll::new(6, "p3".into());
        assert_eq!(r.get_roll(), 6);
    }
}
