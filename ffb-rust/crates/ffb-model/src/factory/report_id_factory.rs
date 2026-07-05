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
}
