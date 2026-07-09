use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportCloudBurster.java`.
#[derive(Debug, Clone)]
pub struct ReportCloudBurster {
    pub thrower_id: Option<String>,
    pub interceptor_id: Option<String>,
    pub thrower_team_id: Option<String>,
}

impl ReportCloudBurster {
    pub fn new(
        thrower_id: Option<String>,
        interceptor_id: Option<String>,
        thrower_team_id: Option<String>,
    ) -> Self {
        Self { thrower_id, interceptor_id, thrower_team_id }
    }

    pub fn get_thrower_id(&self) -> Option<&str> { self.thrower_id.as_deref() }
    pub fn get_interceptor_id(&self) -> Option<&str> { self.interceptor_id.as_deref() }
    pub fn get_thrower_team_id(&self) -> Option<&str> { self.thrower_team_id.as_deref() }
}

impl IReport for ReportCloudBurster {
    fn get_id(&self) -> ReportId { ReportId::CLOUD_BURSTER }
}

impl ReportCloudBurster {
    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "throwerId": self.thrower_id,
            "interceptorId": self.interceptor_id,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            thrower_id: json["throwerId"].as_str().map(str::to_string),
            interceptor_id: json["interceptorId"].as_str().map(str::to_string),
            thrower_team_id: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportCloudBurster {
        ReportCloudBurster::new(Some("t1".into()), Some("i1".into()), Some("team1".into()))
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::CLOUD_BURSTER); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "cloudBurster"); }

    #[test]
    fn get_thrower_id() { assert_eq!(make().get_thrower_id(), Some("t1")); }

    #[test]
    fn get_interceptor_id() { assert_eq!(make().get_interceptor_id(), Some("i1")); }

    #[test]
    fn get_thrower_team_id() { assert_eq!(make().get_thrower_team_id(), Some("team1")); }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportCloudBurster::from_json(&json);
        assert_eq!(restored.thrower_id, original.thrower_id);
        assert_eq!(restored.interceptor_id, original.interceptor_id);
        // thrower_team_id is not serialized (not in Java toJsonValue)
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("cloudBurster"));
    }
}
