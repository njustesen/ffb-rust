use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;
use crate::model::player_state::PlayerState;

/// 1:1 translation of `ReportApothecaryChoice.java`.
#[derive(Debug, Clone)]
pub struct ReportApothecaryChoice {
    pub player_id: String,
    pub player_state: PlayerState,
    /// `SeriousInjury` stored as its enum name; `None` if null.
    pub serious_injury: Option<String>,
}

impl ReportApothecaryChoice {
    pub fn new(
        player_id: String,
        player_state: PlayerState,
        serious_injury: Option<String>,
    ) -> Self {
        Self { player_id, player_state, serious_injury }
    }

    pub fn get_player_id(&self) -> &str { &self.player_id }
    pub fn get_player_state(&self) -> &PlayerState { &self.player_state }
    pub fn get_serious_injury(&self) -> Option<&str> { self.serious_injury.as_deref() }
}

impl IReport for ReportApothecaryChoice {
    fn get_id(&self) -> ReportId { ReportId::APOTHECARY_CHOICE }
}

impl ReportApothecaryChoice {
    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "playerId": self.player_id,
            "playerState": serde_json::Value::Null,
            "seriousInjury": self.serious_injury,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            player_id: json["playerId"].as_str().unwrap_or("").to_string(),
            player_state: PlayerState::new(),
            serious_injury: json["seriousInjury"].as_str().map(str::to_string),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportApothecaryChoice {
        ReportApothecaryChoice::new("p1".into(), PlayerState::new(), Some("BROKEN_RIBS".into()))
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::APOTHECARY_CHOICE);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "apothecaryChoice");
    }

    #[test]
    fn get_serious_injury() {
        assert_eq!(make().get_serious_injury(), Some("BROKEN_RIBS"));
    }

    #[test]
    fn get_player_id() {
        assert_eq!(make().get_player_id(), "p1");
    }

    #[test]
    fn no_serious_injury() {
        let r = ReportApothecaryChoice::new("p2".into(), PlayerState::new(), None);
        assert_eq!(r.get_serious_injury(), None);
        assert_eq!(r.get_player_id(), "p2");
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportApothecaryChoice::from_json(&json);
        assert_eq!(restored.player_id, original.player_id);
        assert_eq!(restored.serious_injury, original.serious_injury);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("apothecaryChoice"));
    }
}
