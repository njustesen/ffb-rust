use crate::enums::PlayerState;
use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportApothecaryRoll.java`.
#[derive(Debug, Clone)]
pub struct ReportApothecaryRoll {
    pub player_id: Option<String>,
    pub casualty_roll: Vec<i32>,
    pub player_state: Option<PlayerState>,
    pub serious_injury: Option<String>,
    pub original_injury: Option<String>,
    pub casualty_modifiers: Vec<String>,
}

impl ReportApothecaryRoll {
    pub fn new(
        player_id: Option<String>,
        casualty_roll: Vec<i32>,
        player_state: Option<PlayerState>,
        serious_injury: Option<String>,
        original_injury: Option<String>,
        casualty_modifiers: Vec<String>,
    ) -> Self {
        Self { player_id, casualty_roll, player_state, serious_injury, original_injury, casualty_modifiers }
    }

    pub fn get_player_id(&self) -> Option<&str> { self.player_id.as_deref() }
    pub fn get_casualty_roll(&self) -> &[i32] { &self.casualty_roll }
    pub fn get_player_state(&self) -> Option<PlayerState> { self.player_state }
    pub fn get_serious_injury(&self) -> Option<&str> { self.serious_injury.as_deref() }
    pub fn get_original_injury(&self) -> Option<&str> { self.original_injury.as_deref() }
    pub fn get_casualty_modifiers(&self) -> &[String] { &self.casualty_modifiers }
}

impl IReport for ReportApothecaryRoll {
    fn get_id(&self) -> ReportId { ReportId::APOTHECARY_ROLL }
}

impl ReportApothecaryRoll {
    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "playerId": self.player_id,
            "casualtyRoll": self.casualty_roll,
            "playerState": self.player_state.map(|ps| ps.0),
            "seriousInjury": self.serious_injury,
            "seriousInjuryOld": self.original_injury,
            "casualtyModifiers": self.casualty_modifiers,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            player_id: json["playerId"].as_str().map(str::to_string),
            casualty_roll: json["casualtyRoll"].as_array().map(|a| a.iter().map(|v| v.as_i64().unwrap_or(0) as i32).collect()).unwrap_or_default(),
            player_state: json["playerState"].as_u64().map(|v| PlayerState(v as u32)),
            serious_injury: json["seriousInjury"].as_str().map(str::to_string),
            original_injury: json["seriousInjuryOld"].as_str().map(str::to_string),
            casualty_modifiers: json["casualtyModifiers"].as_array().map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_string)).collect()).unwrap_or_default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportApothecaryRoll {
        ReportApothecaryRoll::new(
            Some("p1".into()),
            vec![3, 4],
            None,
            Some("BROKEN_RIBS".into()),
            None,
            vec![],
        )
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::APOTHECARY_ROLL); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "apothecaryRoll"); }

    #[test]
    fn get_serious_injury() { assert_eq!(make().get_serious_injury(), Some("BROKEN_RIBS")); }

    #[test]
    fn get_player_id_and_casualty_roll() {
        assert_eq!(make().get_player_id(), Some("p1"));
        assert_eq!(make().get_casualty_roll(), &[3, 4]);
    }

    #[test]
    fn get_casualty_modifiers_and_original_injury() {
        assert_eq!(make().get_casualty_modifiers(), &[] as &[String]);
        assert_eq!(make().get_original_injury(), None);
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportApothecaryRoll::from_json(&json);
        assert_eq!(restored.player_id, original.player_id);
        assert_eq!(restored.casualty_roll, original.casualty_roll);
        assert_eq!(restored.player_state, original.player_state);
        assert_eq!(restored.serious_injury, original.serious_injury);
        assert_eq!(restored.original_injury, original.original_injury);
        assert_eq!(restored.casualty_modifiers, original.casualty_modifiers);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("apothecaryRoll"));
    }
}
