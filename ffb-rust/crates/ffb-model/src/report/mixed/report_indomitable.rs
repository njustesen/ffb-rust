use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportIndomitable.java`.
#[derive(Debug, Clone)]
pub struct ReportIndomitable {
    pub player_id: Option<String>,
    pub defender_id: Option<String>,
}

impl ReportIndomitable {
    pub fn new(player_id: Option<String>, defender_id: Option<String>) -> Self {
        Self { player_id, defender_id }
    }

    pub fn get_player_id(&self) -> Option<&str> { self.player_id.as_deref() }
    pub fn get_defender_id(&self) -> Option<&str> { self.defender_id.as_deref() }
}

impl IReport for ReportIndomitable {
    fn get_id(&self) -> ReportId { ReportId::INDOMITABLE }
}

impl ReportIndomitable {
    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "defenderId": self.defender_id,
            "playerId": self.player_id,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            player_id: json["playerId"].as_str().map(str::to_string),
            defender_id: json["defenderId"].as_str().map(str::to_string),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportIndomitable {
        ReportIndomitable::new(Some("p1".into()), Some("d1".into()))
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::INDOMITABLE); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "indomitable"); }

    #[test]
    fn get_player_id() { assert_eq!(make().get_player_id(), Some("p1")); }

    #[test]
    fn get_defender_id() { assert_eq!(make().get_defender_id(), Some("d1")); }

    #[test]
    fn none_ids() {
        let r = ReportIndomitable::new(None, None);
        assert_eq!(r.get_player_id(), None);
        assert_eq!(r.get_defender_id(), None);
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportIndomitable::from_json(&json);
        assert_eq!(restored.player_id, original.player_id);
        assert_eq!(restored.defender_id, original.defender_id);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("indomitable"));
    }
}
