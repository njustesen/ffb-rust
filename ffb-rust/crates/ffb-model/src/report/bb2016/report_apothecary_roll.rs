use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;
use crate::enums::PlayerState;

/// 1:1 translation of `ReportApothecaryRoll.java`.
#[derive(Debug, Clone)]
pub struct ReportApothecaryRoll {
    pub player_id: String,
    pub casualty_roll: Vec<i32>,
    pub player_state: Option<PlayerState>,
    pub serious_injury: Option<String>,
}

impl ReportApothecaryRoll {
    pub fn new(
        player_id: String,
        casualty_roll: Vec<i32>,
        player_state: Option<PlayerState>,
        serious_injury: Option<String>,
    ) -> Self {
        Self { player_id, casualty_roll, player_state, serious_injury }
    }

    pub fn get_player_id(&self) -> &str { &self.player_id }
    pub fn get_casualty_roll(&self) -> &[i32] { &self.casualty_roll }
    pub fn get_player_state(&self) -> Option<PlayerState> { self.player_state }
    pub fn get_serious_injury(&self) -> Option<&str> { self.serious_injury.as_deref() }

    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "playerId": self.player_id,
            "casualtyRoll": self.casualty_roll,
            "playerState": self.player_state.map(|ps| ps.id()),
            "seriousInjury": self.serious_injury,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            player_id: json["playerId"].as_str().unwrap_or("").to_string(),
            casualty_roll: json["casualtyRoll"].as_array().map(|a| a.iter().map(|v| v.as_i64().unwrap_or(0) as i32).collect()).unwrap_or_default(),
            player_state: json["playerState"].as_u64().map(|n| PlayerState::new(n as u32)),
            serious_injury: json["seriousInjury"].as_str().map(str::to_string),
        }
    }
}

impl IReport for ReportApothecaryRoll {
    fn get_id(&self) -> ReportId { ReportId::APOTHECARY_ROLL }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportApothecaryRoll {
        ReportApothecaryRoll::new("p1".into(), vec![3, 4], None, None)
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::APOTHECARY_ROLL);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "apothecaryRoll");
    }

    #[test]
    fn fields() {
        let r = make();
        assert_eq!(r.get_player_id(), "p1");
        assert_eq!(r.get_casualty_roll(), &[3, 4]);
        assert_eq!(r.get_serious_injury(), None);
    }

    #[test]
    fn serious_injury_stored() {
        let r = ReportApothecaryRoll::new("p2".into(), vec![5, 6], None, Some("NIGGLING_INJURY".into()));
        assert_eq!(r.get_serious_injury(), Some("NIGGLING_INJURY"));
    }

    #[test]
    fn player_state_none_by_default() {
        let r = make();
        assert!(r.get_player_state().is_none());
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportApothecaryRoll::from_json(&json);
        assert_eq!(restored.player_id, original.player_id);
        assert_eq!(restored.casualty_roll, original.casualty_roll);
        assert_eq!(restored.serious_injury, original.serious_injury);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("apothecaryRoll"));
    }
}
