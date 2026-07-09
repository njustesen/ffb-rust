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

impl ReportHandOver {
    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "catcherId": self.catcher_id,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            catcher_id: json["catcherId"].as_str().unwrap_or("").to_string(),
        }
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

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportHandOver::from_json(&json);
        assert_eq!(restored.catcher_id, original.catcher_id);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("handOver"));
    }
}
