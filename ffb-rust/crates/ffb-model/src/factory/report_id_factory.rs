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
