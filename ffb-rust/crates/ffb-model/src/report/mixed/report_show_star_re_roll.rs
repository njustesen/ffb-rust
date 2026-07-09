use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportShowStarReRoll.java`.
#[derive(Debug, Clone)]
pub struct ReportShowStarReRoll {
    /// `player` — player id.
    pub player_id: Option<String>,
}

impl ReportShowStarReRoll {
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

impl IReport for ReportShowStarReRoll {
    fn get_id(&self) -> ReportId { ReportId::SHOW_STAR_RE_ROLL }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportShowStarReRoll {
        ReportShowStarReRoll::new(Some("p1".into()))
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::SHOW_STAR_RE_ROLL); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "showStarReRoll"); }

    #[test]
    fn get_player_id() { assert_eq!(make().get_player_id(), Some("p1")); }

    #[test]
    fn player_id_none() {
        let r = ReportShowStarReRoll::new(None);
        assert!(r.get_player_id().is_none());
    }

    #[test]
    fn different_player_id() {
        let r = ReportShowStarReRoll::new(Some("star1".into()));
        assert_eq!(r.get_player_id(), Some("star1"));
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportShowStarReRoll::from_json(&json);
        assert_eq!(restored.player_id, original.player_id);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("showStarReRoll"));
    }
}
