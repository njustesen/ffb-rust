use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportTimeoutEnforced.java`.
#[derive(Debug, Clone)]
pub struct ReportTimeoutEnforced {
    pub coach: String,
}

impl ReportTimeoutEnforced {
    pub fn new(coach: String) -> Self {
        Self { coach }
    }

    pub fn get_coach(&self) -> &str { &self.coach }
}

impl IReport for ReportTimeoutEnforced {
    fn get_id(&self) -> ReportId { ReportId::TIMEOUT_ENFORCED }
}

impl ReportTimeoutEnforced {
    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "coach": self.coach,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            coach: json["coach"].as_str().unwrap_or("").to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportTimeoutEnforced {
        ReportTimeoutEnforced::new("Coach McCoach".into())
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportTimeoutEnforced::from_json(&json);
        assert_eq!(restored.coach, original.coach);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("timeoutEnforced"));
    }
}
