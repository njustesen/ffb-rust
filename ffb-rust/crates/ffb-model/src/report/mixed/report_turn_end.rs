use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// Knockout recovery entry — mirrors `KnockoutRecovery` from Java.
#[derive(Debug, Clone)]
pub struct KnockoutRecovery {
    pub player_id: String,
    pub recovered: bool,
}

impl KnockoutRecovery {
    pub fn new(player_id: String, recovered: bool) -> Self {
        Self { player_id, recovered }
    }

    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "playerId": self.player_id,
            "recovering": self.recovered,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            player_id: json["playerId"].as_str().unwrap_or("").to_string(),
            recovered: json["recovering"].as_bool().unwrap_or(false),
        }
    }
}

/// Heat exhaustion entry — mirrors `HeatExhaustion` from Java.
#[derive(Debug, Clone)]
pub struct HeatExhaustion {
    pub player_id: String,
    pub roll: i32,
}

impl HeatExhaustion {
    pub fn new(player_id: String, roll: i32) -> Self {
        Self { player_id, roll }
    }

    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "playerId": self.player_id,
            "exhausted": true,
            "roll": self.roll,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            player_id: json["playerId"].as_str().unwrap_or("").to_string(),
            roll: json["roll"].as_i64().unwrap_or(0) as i32,
        }
    }
}

/// 1:1 translation of `ReportTurnEnd.java`.
#[derive(Debug, Clone)]
pub struct ReportTurnEnd {
    /// `fPlayerIdTouchdown`
    pub player_id_touchdown: Option<String>,
    /// `fKnockoutRecoveries`
    pub knockout_recoveries: Vec<KnockoutRecovery>,
    /// `fHeatExhaustions`
    pub heat_exhaustions: Vec<HeatExhaustion>,
    /// `unzappedPlayers` — player ids of unzapped players.
    pub unzapped_players: Vec<String>,
    /// `heatRoll`
    pub heat_roll: i32,
}

impl ReportTurnEnd {
    pub fn new(
        player_id_touchdown: Option<String>,
        knockout_recoveries: Vec<KnockoutRecovery>,
        heat_exhaustions: Vec<HeatExhaustion>,
        unzapped_players: Vec<String>,
        heat_roll: i32,
    ) -> Self {
        Self { player_id_touchdown, knockout_recoveries, heat_exhaustions, unzapped_players, heat_roll }
    }

    pub fn get_player_id_touchdown(&self) -> Option<&str> { self.player_id_touchdown.as_deref() }
    pub fn get_knockout_recoveries(&self) -> &[KnockoutRecovery] { &self.knockout_recoveries }
    pub fn get_heat_exhaustions(&self) -> &[HeatExhaustion] { &self.heat_exhaustions }
    pub fn get_unzapped_players(&self) -> &[String] { &self.unzapped_players }
    pub fn get_heat_roll(&self) -> i32 { self.heat_roll }

    pub fn to_json_value(&self) -> serde_json::Value {
        let knockout_array: Vec<serde_json::Value> = self.knockout_recoveries
            .iter()
            .map(|k| k.to_json_value())
            .collect();
        let heat_array: Vec<serde_json::Value> = self.heat_exhaustions
            .iter()
            .map(|h| h.to_json_value())
            .collect();
        let unzap_array: Vec<serde_json::Value> = self.unzapped_players
            .iter()
            .map(|id| serde_json::json!({ "playerId": id }))
            .collect();
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "playerIdTouchdown": self.player_id_touchdown,
            "knockoutRecoveryArray": knockout_array,
            "heatExhaustionArray": heat_array,
            "unzapArray": unzap_array,
            "heatRoll": self.heat_roll,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        let knockout_recoveries = json["knockoutRecoveryArray"]
            .as_array()
            .map(|a| a.iter().map(KnockoutRecovery::from_json).collect())
            .unwrap_or_default();
        let heat_exhaustions = json["heatExhaustionArray"]
            .as_array()
            .map(|a| a.iter().map(HeatExhaustion::from_json).collect())
            .unwrap_or_default();
        let unzapped_players = json["unzapArray"]
            .as_array()
            .map(|a| a.iter().filter_map(|v| v["playerId"].as_str().map(str::to_string)).collect())
            .unwrap_or_default();
        Self {
            player_id_touchdown: json["playerIdTouchdown"].as_str().map(str::to_string),
            knockout_recoveries,
            heat_exhaustions,
            unzapped_players,
            heat_roll: json["heatRoll"].as_i64().unwrap_or(0) as i32,
        }
    }
}

impl IReport for ReportTurnEnd {
    fn get_id(&self) -> ReportId { ReportId::TURN_END }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportTurnEnd {
        ReportTurnEnd::new(
            Some("scorer".into()),
            vec![KnockoutRecovery::new("ko1".into(), true)],
            vec![],
            vec![],
            0,
        )
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::TURN_END); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "turnEnd"); }

    #[test]
    fn get_player_id_touchdown() { assert_eq!(make().get_player_id_touchdown(), Some("scorer")); }

    #[test]
    fn get_knockout_recoveries() { assert_eq!(make().get_knockout_recoveries().len(), 1); }
    #[test]
    fn get_name_is_nonempty() {
        assert!(!make().get_name().is_empty());
    }

    #[test]
    fn serialization_round_trip() {
        let original = ReportTurnEnd::new(
            Some("scorer".into()),
            vec![KnockoutRecovery::new("ko1".into(), true)],
            vec![HeatExhaustion::new("he1".into(), 3)],
            vec!["unzap1".into()],
            5,
        );
        let json = original.to_json_value();
        let restored = ReportTurnEnd::from_json(&json);
        assert_eq!(restored.player_id_touchdown, original.player_id_touchdown);
        assert_eq!(restored.knockout_recoveries.len(), 1);
        assert_eq!(restored.knockout_recoveries[0].player_id, "ko1");
        assert!(restored.knockout_recoveries[0].recovered);
        assert_eq!(restored.heat_exhaustions.len(), 1);
        assert_eq!(restored.heat_exhaustions[0].player_id, "he1");
        assert_eq!(restored.heat_exhaustions[0].roll, 3);
        assert_eq!(restored.unzapped_players, vec!["unzap1".to_string()]);
        assert_eq!(restored.heat_roll, 5);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("turnEnd"));
    }
}
