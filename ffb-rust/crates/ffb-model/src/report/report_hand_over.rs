use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportHandOver.java`.
#[derive(Debug, Clone)]
pub struct ReportHandOver {
    /// Translated from `fCatcherId`.
    pub catcher_id: String,
}

impl ReportHandOver {
    pub fn new(catcher_id: String) -> Self {
        Self { catcher_id }
    }

    pub fn get_catcher_id(&self) -> &str {
        &self.catcher_id
    }
}

impl IReport for ReportHandOver {
    fn get_id(&self) -> ReportId {
        ReportId::HAND_OVER
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportHandOver {
        ReportHandOver::new("catcher1".into())
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::HAND_OVER);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "handOver");
    }

    #[test]
    fn catcher_id_getter() {
        assert_eq!(make().get_catcher_id(), "catcher1");
    }

    #[test]
    fn different_catcher_id() {
        let r = ReportHandOver::new("catcher99".into());
        assert_eq!(r.get_catcher_id(), "catcher99");
    }

    #[test]
    fn empty_catcher_id() {
        let r = ReportHandOver::new(String::new());
        assert_eq!(r.get_catcher_id(), "");
    }
}
