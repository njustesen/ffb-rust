use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `UtilReport.java`.
pub struct UtilReport;

impl UtilReport {
    /// Java: validateReportId(IReport, ReportId) — panics if `report` is missing or
    /// its id doesn't match `received_id`.
    pub fn validate_report_id(report: Option<&dyn IReport>, received_id: Option<ReportId>) {
        let report = report.expect("Parameter report must not be null.");
        if Some(report.get_id()) != received_id {
            panic!(
                "Wrong report id. Expected {} received {}",
                report.get_id().get_name(),
                received_id.map(|id| id.get_name().to_string()).unwrap_or_else(|| "null".to_string())
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DummyReport(ReportId);
    impl IReport for DummyReport {
        fn get_id(&self) -> ReportId {
            self.0
        }
    }

    #[test]
    fn matching_id_does_not_panic() {
        let report = DummyReport(ReportId::INJURY);
        UtilReport::validate_report_id(Some(&report), Some(ReportId::INJURY));
    }

    #[test]
    #[should_panic(expected = "Parameter report must not be null.")]
    fn null_report_panics() {
        UtilReport::validate_report_id(None, Some(ReportId::INJURY));
    }

    #[test]
    #[should_panic(expected = "Wrong report id")]
    fn mismatched_id_panics() {
        let report = DummyReport(ReportId::INJURY);
        UtilReport::validate_report_id(Some(&report), Some(ReportId::DODGE_ROLL));
    }
}
