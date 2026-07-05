use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportDoubleHiredStaff.java`.
#[derive(Debug, Clone)]
pub struct ReportDoubleHiredStaff {
    pub staff_name: Option<String>,
}

impl ReportDoubleHiredStaff {
    pub fn new(staff_name: Option<String>) -> Self {
        Self { staff_name }
    }

    pub fn get_staff_name(&self) -> Option<&str> { self.staff_name.as_deref() }
}

impl IReport for ReportDoubleHiredStaff {
    fn get_id(&self) -> ReportId { ReportId::DOUBLE_HIRED_STAFF }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportDoubleHiredStaff {
        ReportDoubleHiredStaff::new(Some("apothecary".into()))
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::DOUBLE_HIRED_STAFF); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "doubleHiredStaff"); }

    #[test]
    fn get_staff_name() { assert_eq!(make().get_staff_name(), Some("apothecary")); }
}
