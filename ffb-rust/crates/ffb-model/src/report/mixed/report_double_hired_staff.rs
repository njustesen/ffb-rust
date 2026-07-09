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

impl ReportDoubleHiredStaff {
    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "name": self.staff_name,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            staff_name: json["name"].as_str().map(str::to_string),
        }
    }
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

    #[test]
    fn none_staff_name() {
        let r = ReportDoubleHiredStaff::new(None);
        assert_eq!(r.get_staff_name(), None);
    }

    #[test]
    fn different_staff_name() {
        let r = ReportDoubleHiredStaff::new(Some("cheerleader".into()));
        assert_eq!(r.get_staff_name(), Some("cheerleader"));
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportDoubleHiredStaff::from_json(&json);
        assert_eq!(restored.staff_name, original.staff_name);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("doubleHiredStaff"));
    }
}
