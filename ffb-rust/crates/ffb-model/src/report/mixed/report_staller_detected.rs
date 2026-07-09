use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportStallerDetected.java`.
#[derive(Debug, Clone)]
pub struct ReportStallerDetected {
    /// `fPlayerId`
    pub player_id: Option<String>,
}

impl ReportStallerDetected {
    pub fn new(player_id: Option<String>) -> Self {
        Self { player_id }
    }

    pub fn get_player_id(&self) -> Option<&str> { self.player_id.as_deref() }

    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "playerId": self.player_id,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            player_id: json["playerId"].as_str().map(str::to_string),
        }
    }
}

impl IReport for ReportStallerDetected {
    fn get_id(&self) -> ReportId { ReportId::STALLER_DETECTED }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportStallerDetected {
        ReportStallerDetected::new(Some("p1".into()))
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::STALLER_DETECTED); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "stallerDetected"); }

    #[test]
    fn get_player_id() { assert_eq!(make().get_player_id(), Some("p1")); }

    #[test]
    fn player_id_none() {
        let r = ReportStallerDetected::new(None);
        assert!(r.get_player_id().is_none());
    }

    #[test]
    fn different_player_id() {
        let r = ReportStallerDetected::new(Some("staller42".into()));
        assert_eq!(r.get_player_id(), Some("staller42"));
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportStallerDetected::from_json(&json);
        assert_eq!(restored.player_id, original.player_id);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("stallerDetected"));
    }
}
