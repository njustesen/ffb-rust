use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportChompRemoved.java`.
#[derive(Debug, Clone)]
pub struct ReportChompRemoved {
    pub player: String,
    pub successful: bool,
}

impl ReportChompRemoved {
    pub fn new(player: String, successful: bool) -> Self {
        Self { player, successful }
    }

    pub fn get_player(&self) -> &str { &self.player }
    pub fn is_successful(&self) -> bool { self.successful }
}

impl IReport for ReportChompRemoved {
    fn get_id(&self) -> ReportId { ReportId::CHOMP_REMOVED }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_id() {
        assert_eq!(ReportChompRemoved::new("p1".into(), true).get_id(), ReportId::CHOMP_REMOVED);
    }

    #[test]
    fn get_name() {
        assert_eq!(ReportChompRemoved::new("p1".into(), true).get_name(), "chompRemoved");
    }

    #[test]
    fn fields() {
        let r = ReportChompRemoved::new("p1".into(), true);
        assert_eq!(r.get_player(), "p1");
        assert!(r.is_successful());
    }
}
