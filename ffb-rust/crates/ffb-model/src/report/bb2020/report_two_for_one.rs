use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportTwoForOne.java`.
#[derive(Debug, Clone)]
pub struct ReportTwoForOne {
    pub player_id: String,
    pub partner_id: String,
    pub used: bool,
}

impl ReportTwoForOne {
    pub fn new(player_id: String, partner_id: String, used: bool) -> Self {
        Self { player_id, partner_id, used }
    }

    pub fn get_player_id(&self) -> &str { &self.player_id }
    pub fn get_partner_id(&self) -> &str { &self.partner_id }
    pub fn is_used(&self) -> bool { self.used }
}

impl IReport for ReportTwoForOne {
    fn get_id(&self) -> ReportId { ReportId::TWO_FOR_ONE }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportTwoForOne {
        ReportTwoForOne::new("p1".into(), "p2".into(), true)
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::TWO_FOR_ONE);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "twoForOne");
    }

    #[test]
    fn fields() {
        let r = make();
        assert_eq!(r.get_player_id(), "p1");
        assert_eq!(r.get_partner_id(), "p2");
        assert!(r.is_used());
    }

    #[test]
    fn not_used() {
        let r = ReportTwoForOne::new("p3".into(), "p4".into(), false);
        assert!(!r.is_used());
        assert_eq!(r.get_player_id(), "p3");
    }

    #[test]
    fn partner_id() {
        let r = ReportTwoForOne::new("a".into(), "b".into(), true);
        assert_eq!(r.get_partner_id(), "b");
    }
}
