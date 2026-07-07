use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportPilingOn.java`.
#[derive(Debug, Clone)]
pub struct ReportPilingOn {
    pub player_id: String,
    pub used: bool,
    pub re_roll_injury: bool,
}

impl ReportPilingOn {
    pub fn new(player_id: String, used: bool, re_roll_injury: bool) -> Self {
        Self { player_id, used, re_roll_injury }
    }

    pub fn get_player_id(&self) -> &str { &self.player_id }
    pub fn is_used(&self) -> bool { self.used }
    pub fn is_re_roll_injury(&self) -> bool { self.re_roll_injury }
}

impl IReport for ReportPilingOn {
    fn get_id(&self) -> ReportId { ReportId::PILING_ON }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportPilingOn {
        ReportPilingOn::new("p1".into(), true, false)
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::PILING_ON);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "pilingOn");
    }

    #[test]
    fn fields() {
        let r = make();
        assert_eq!(r.get_player_id(), "p1");
        assert!(r.is_used());
        assert!(!r.is_re_roll_injury());
    }

    #[test]
    fn not_used() {
        let r = ReportPilingOn::new("p2".into(), false, false);
        assert!(!r.is_used());
        assert_eq!(r.get_player_id(), "p2");
    }

    #[test]
    fn re_roll_injury_flag() {
        let r = ReportPilingOn::new("p3".into(), true, true);
        assert!(r.is_re_roll_injury());
    }
}
