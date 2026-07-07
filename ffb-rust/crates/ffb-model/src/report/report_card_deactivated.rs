use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportCardDeactivated.java`.
/// `Card` is stored as its name string (card type name).
#[derive(Debug, Clone)]
pub struct ReportCardDeactivated {
    /// Card type name (replaces `Card` object).
    pub card: String,
}

impl ReportCardDeactivated {
    pub fn new(card: String) -> Self {
        Self { card }
    }

    pub fn get_card(&self) -> &str { &self.card }
}

impl IReport for ReportCardDeactivated {
    fn get_id(&self) -> ReportId { ReportId::CARD_DEACTIVATED }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportCardDeactivated {
        ReportCardDeactivated::new("CUSTARD_PIE".into())
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::CARD_DEACTIVATED);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "cardDeactivated");
    }

    #[test]
    fn get_card() {
        assert_eq!(make().get_card(), "CUSTARD_PIE");
    }

    #[test]
    fn different_card() {
        let r = ReportCardDeactivated::new("ILLEGAL_PROCEDURE".into());
        assert_eq!(r.get_card(), "ILLEGAL_PROCEDURE");
    }

    #[test]
    fn card_matches_field() {
        let r = make();
        assert_eq!(r.get_card(), r.card.as_str());
    }
}
