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
}
