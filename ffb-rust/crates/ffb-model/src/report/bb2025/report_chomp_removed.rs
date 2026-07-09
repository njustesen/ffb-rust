use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportChompRemoved.java`.
#[derive(Debug, Clone)]
pub struct ReportChompRemoved {
    pub player: String,
    pub successful: bool,
}

impl ReportChompRemoved {
    pub fn new(player: String, successful: bool) -> Self {
        Self { player, successful }
    }

    pub fn get_player(&self) -> &str { &self.player }
    pub fn is_successful(&self) -> bool { self.successful }

    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "playerId": self.player,
            "successful": self.successful,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            player: json["playerId"].as_str().unwrap_or("").to_string(),
            successful: json["successful"].as_bool().unwrap_or(false),
        }
    }
}

impl IReport for ReportChompRemoved {
    fn get_id(&self) -> ReportId { ReportId::CHOMP_REMOVED }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_id() {
        assert_eq!(ReportChompRemoved::new("p1".into(), true).get_id(), ReportId::CHOMP_REMOVED);
    }

    #[test]
    fn get_name() {
        assert_eq!(ReportChompRemoved::new("p1".into(), true).get_name(), "chompRemoved");
    }

    #[test]
    fn fields() {
        let r = ReportChompRemoved::new("p1".into(), true);
        assert_eq!(r.get_player(), "p1");
        assert!(r.is_successful());
    }

    #[test]
    fn unsuccessful() {
        let r = ReportChompRemoved::new("p2".into(), false);
        assert!(!r.is_successful());
        assert_eq!(r.get_player(), "p2");
    }

    #[test]
    fn different_player() {
        let r = ReportChompRemoved::new("bigGuy".into(), true);
        assert_eq!(r.get_player(), "bigGuy");
    }

    #[test]
    fn serialization_round_trip() {
        let original = ReportChompRemoved::new("p1".into(), true);
        let json = original.to_json_value();
        let restored = ReportChompRemoved::from_json(&json);
        assert_eq!(restored.player, original.player);
        assert_eq!(restored.successful, original.successful);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = ReportChompRemoved::new("p1".into(), true).to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("chompRemoved"));
    }
}
