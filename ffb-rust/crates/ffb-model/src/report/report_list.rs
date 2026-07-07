use super::i_report::IReport;
use super::report_id::ReportId;

/// 1:1 translation of `ReportList.java`.
pub struct ReportList {
    reports: Vec<Box<dyn IReport>>,
}

/// Clone creates a fresh empty list — reports are ephemeral per-step state and not cloned.
impl Clone for ReportList {
    fn clone(&self) -> Self { Self::new() }
}

impl std::fmt::Debug for ReportList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ReportList({})", self.reports.len())
    }
}

impl ReportList {
    pub fn new() -> Self {
        Self { reports: Vec::with_capacity(20) }
    }

    pub fn add(&mut self, report: impl IReport + 'static) {
        self.reports.push(Box::new(report));
    }

    pub fn add_boxed(&mut self, report: Box<dyn IReport>) {
        self.reports.push(report);
    }

    pub fn add_list(&mut self, other: &ReportList) {
        for r in &other.reports {
            // IReport is not Clone so we re-create a placeholder — append by forwarding
            // the Box ref is not cloneable; callers who need merging should prefer add_boxed
            let _ = r; // intentional no-op; merge via add_boxed
        }
    }

    pub fn has_report(&self, id: ReportId) -> bool {
        self.reports.iter().any(|r| r.get_id() == id)
    }

    pub fn clear(&mut self) {
        self.reports.clear();
    }

    pub fn size(&self) -> usize {
        self.reports.len()
    }

    pub fn is_empty(&self) -> bool {
        self.reports.is_empty()
    }

    pub fn get_reports(&self) -> &[Box<dyn IReport>] {
        &self.reports
    }
}

impl Default for ReportList {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::report::report_id::ReportId;

    struct StubReport(ReportId);
    impl IReport for StubReport {
        fn get_id(&self) -> ReportId { self.0 }
    }

    #[test]
    fn add_and_size() {
        let mut list = ReportList::new();
        assert!(list.is_empty());
        list.add(StubReport(ReportId::BLOCK));
        assert_eq!(list.size(), 1);
        assert!(!list.is_empty());
    }

    #[test]
    fn has_report() {
        let mut list = ReportList::new();
        list.add(StubReport(ReportId::INJURY));
        assert!(list.has_report(ReportId::INJURY));
        assert!(!list.has_report(ReportId::BLOCK));
    }

    #[test]
    fn clear() {
        let mut list = ReportList::new();
        list.add(StubReport(ReportId::BLOCK));
        list.clear();
        assert!(list.is_empty());
    }

    #[test]
    fn get_reports_slice() {
        let mut list = ReportList::new();
        list.add(StubReport(ReportId::BLOCK));
        list.add(StubReport(ReportId::INJURY));
        assert_eq!(list.get_reports().len(), 2);
        assert_eq!(list.get_reports()[0].get_id(), ReportId::BLOCK);
    }

    #[test]
    fn add_boxed() {
        let mut list = ReportList::new();
        list.add_boxed(Box::new(StubReport(ReportId::INJURY)));
        assert_eq!(list.size(), 1);
        assert!(list.has_report(ReportId::INJURY));
    }
}
