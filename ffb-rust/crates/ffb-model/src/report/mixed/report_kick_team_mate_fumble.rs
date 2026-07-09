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
    fn get_id() { assert_eq!(make().get_id(), ReportId::KICK_TEAM_MATE_FUMBLE); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "kickTeamMateFumble"); }

    #[test]
    fn is_default() {
        let r = ReportKickTeamMateFumble::default();
        assert_eq!(r.get_id(), ReportId::KICK_TEAM_MATE_FUMBLE);
    }

    #[test]
    fn new_and_default_equal() {
        let via_new = ReportKickTeamMateFumble::new();
        let via_default = ReportKickTeamMateFumble::default();
        assert_eq!(via_new.get_id(), via_default.get_id());
    }

    #[test]
    fn unit_struct_has_no_fields() {
        let r = make();
        assert_eq!(r.get_name(), "kickTeamMateFumble");
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
