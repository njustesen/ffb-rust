use super::report_id::ReportId;

/// 1:1 translation of `IReport.java`.
pub trait IReport: Send + Sync {
    fn get_id(&self) -> ReportId;
    fn get_name(&self) -> &str {
        self.get_id().get_name()
    }
}
