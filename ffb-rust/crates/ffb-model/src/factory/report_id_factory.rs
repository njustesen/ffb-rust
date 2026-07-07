use crate::enums::ReportId;

/// 1:1 translation of com.fumbbl.ffb.factory.ReportIdFactory.
pub struct ReportIdFactory;

impl Default for ReportIdFactory {
    fn default() -> Self { ReportIdFactory }
}

impl ReportIdFactory {
    pub fn for_name(&self, name: &str) -> Option<ReportId> {
        ReportId::from_name(name)
    }

    pub fn initialize(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn for_name_returns_known_report_id() {
        let f = ReportIdFactory::default();
        assert!(f.for_name("dodgeRoll").is_some());
    }

    #[test]
    fn for_name_unknown_returns_none() {
        assert_eq!(ReportIdFactory::default().for_name("invalid"), None);
    }

    #[test]
    fn for_name_second_known() {
        let f = ReportIdFactory::default();
        assert!(f.for_name("blockRoll").is_some());
    }

    #[test]
    fn initialize_does_not_panic() {
        let mut f = ReportIdFactory::default();
        f.initialize();
    }
    #[test]
    fn for_name_empty_string_returns_none() {
        assert!(ReportIdFactory.for_name("").is_none());
    }
}
