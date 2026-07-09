use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportFumbblResultUpload.java`.
#[derive(Debug, Clone)]
pub struct ReportFumbblResultUpload {
    /// Translated from `fSuccessful`.
    pub successful: bool,
    /// Translated from `fUploadStatus`.
    pub upload_status: String,
}

impl ReportFumbblResultUpload {
    pub fn new(successful: bool, upload_status: String) -> Self {
        Self { successful, upload_status }
    }

    pub fn is_successful(&self) -> bool {
        self.successful
    }

    pub fn get_upload_status(&self) -> &str {
        &self.upload_status
    }
}

impl IReport for ReportFumbblResultUpload {
    fn get_id(&self) -> ReportId {
        ReportId::FUMBBL_RESULT_UPLOAD
    }
}

impl ReportFumbblResultUpload {
    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "successful": self.successful,
            "uploadStatus": self.upload_status,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            successful: json["successful"].as_bool().unwrap_or(false),
            upload_status: json["uploadStatus"].as_str().unwrap_or("").to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportFumbblResultUpload {
        ReportFumbblResultUpload::new(true, "OK".into())
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::FUMBBL_RESULT_UPLOAD);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "fumbblResultUpload");
    }

    #[test]
    fn field_getters() {
        let r = make();
        assert!(r.is_successful());
        assert_eq!(r.get_upload_status(), "OK");
    }

    #[test]
    fn unsuccessful_upload() {
        let r = ReportFumbblResultUpload::new(false, "ERROR".into());
        assert!(!r.is_successful());
        assert_eq!(r.get_upload_status(), "ERROR");
    }

    #[test]
    fn different_status_string() {
        let r = ReportFumbblResultUpload::new(true, "PENDING".into());
        assert!(r.is_successful());
        assert_eq!(r.get_upload_status(), "PENDING");
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportFumbblResultUpload::from_json(&json);
        assert_eq!(restored.successful, original.successful);
        assert_eq!(restored.upload_status, original.upload_status);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("fumbblResultUpload"));
    }
}
