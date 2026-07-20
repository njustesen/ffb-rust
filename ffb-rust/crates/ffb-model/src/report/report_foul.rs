use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportFoul.java`.
#[derive(Debug, Clone)]
pub struct ReportFoul {
    /// Translated from `fDefenderId`.
    pub defender_id: String,
}

impl ReportFoul {
    pub fn new(defender_id: String) -> Self {
        Self { defender_id }
    }

    pub fn get_defender_id(&self) -> &str {
        &self.defender_id
    }
}

impl IReport for ReportFoul {
    fn get_id(&self) -> ReportId {
        ReportId::FOUL
    }
}

impl ReportFoul {
    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "defenderId": self.defender_id,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            defender_id: json["defenderId"].as_str().unwrap_or("").to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportFoul {
        ReportFoul::new("defender1".into())
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportFoul::from_json(&json);
        assert_eq!(restored.defender_id, original.defender_id);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("foul"));
    }
}
