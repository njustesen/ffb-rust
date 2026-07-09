use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportFumblerooskie.java`.
#[derive(Debug, Clone)]
pub struct ReportFumblerooskie {
    pub player_id: Option<String>,
    pub used: bool,
}

impl ReportFumblerooskie {
    pub fn new(player_id: Option<String>, used: bool) -> Self {
        Self { player_id, used }
    }

    pub fn get_player_id(&self) -> Option<&str> { self.player_id.as_deref() }
    pub fn is_used(&self) -> bool { self.used }
}

impl IReport for ReportFumblerooskie {
    fn get_id(&self) -> ReportId { ReportId::FUMBLEROOSKIE }
}

impl ReportFumblerooskie {
    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "used": self.used,
            "playerId": self.player_id,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            player_id: json["playerId"].as_str().map(str::to_string),
            used: json["used"].as_bool().unwrap_or(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportFumblerooskie {
        ReportFumblerooskie::new(Some("p1".into()), true)
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::FUMBLEROOSKIE); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "fumblerooskie"); }

    #[test]
    fn is_used() { assert!(make().is_used()); }

    #[test]
    fn get_player_id() { assert_eq!(make().get_player_id(), Some("p1")); }

    #[test]
    fn not_used_with_none_player() {
        let r = ReportFumblerooskie::new(None, false);
        assert!(!r.is_used());
        assert_eq!(r.get_player_id(), None);
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportFumblerooskie::from_json(&json);
        assert_eq!(restored.player_id, original.player_id);
        assert_eq!(restored.used, original.used);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("fumblerooskie"));
    }
}
