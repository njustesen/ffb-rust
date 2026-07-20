use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportKickTeamMateFumble.java`.
#[derive(Debug, Clone, Default)]
pub struct ReportKickTeamMateFumble;

impl ReportKickTeamMateFumble {
    pub fn new() -> Self {
        Self
    }
}

impl IReport for ReportKickTeamMateFumble {
    fn get_id(&self) -> ReportId { ReportId::KICK_TEAM_MATE_FUMBLE }
}

impl ReportKickTeamMateFumble {
    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
        })
    }

    pub fn from_json(_json: &serde_json::Value) -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportKickTeamMateFumble {
        ReportKickTeamMateFumble::new()
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportKickTeamMateFumble::from_json(&json);
        assert_eq!(restored.get_id(), original.get_id());
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("kickTeamMateFumble"));
    }
}
