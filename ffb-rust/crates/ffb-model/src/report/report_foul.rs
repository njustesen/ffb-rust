use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportFoul.java`.
#[derive(Debug, Clone)]
pub struct ReportFoul {
    /// Translated from `fDefenderId`.
    pub defender_id: String,
}

impl ReportFoul {
    pub fn new(defender_id: String) -> Self {
        Self { defender_id }
    }

    pub fn get_defender_id(&self) -> &str {
        &self.defender_id
    }
}

impl IReport for ReportFoul {
    fn get_id(&self) -> ReportId {
        ReportId::FOUL
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportFoul {
        ReportFoul::new("defender1".into())
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::FOUL);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "foul");
    }

    #[test]
    fn defender_id_getter() {
        assert_eq!(make().get_defender_id(), "defender1");
    }

    #[test]
    fn different_defender_id() {
        let r = ReportFoul::new("defender99".into());
        assert_eq!(r.get_defender_id(), "defender99");
    }

    #[test]
    fn defender_id_matches_field() {
        let r = make();
        assert_eq!(r.get_defender_id(), r.defender_id.as_str());
    }
}
