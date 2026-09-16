use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;
// The BB2016 report carries exactly the same payload entries as the mixed one, so it reuses
// the mixed module's `KnockoutRecovery` / `HeatExhaustion` rather than the unrelated
// `model::{knockout_recovery, heat_exhaustion}` types this file used to import (those made the
// struct un-constructible from the engine, which is why it sat unused).
pub use crate::report::mixed::report_turn_end::{HeatExhaustion, KnockoutRecovery};

/// 1:1 translation of `com.fumbbl.ffb.report.bb2016.ReportTurnEnd`.
///
/// Identical to `report::mixed::report_turn_end::ReportTurnEnd` except that the BB2016 class
/// does NOT write `heatRoll` (`mixed/ReportTurnEnd.toJsonValue` adds `IJsonOption.HEAT_ROLL`,
/// `bb2016/ReportTurnEnd.toJsonValue` stops at `UNZAP_ARRAY`). BB2016 games were constructing
/// the mixed report, so every bb2016 turn end carried a `heatRoll` key Java never writes —
/// one payload diff per turn end, the single highest-volume report in the sweep.
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
}

impl ReportTurnEnd {
    pub fn new(
        player_id_touchdown: Option<String>,
        knockout_recoveries: Vec<KnockoutRecovery>,
        heat_exhaustions: Vec<HeatExhaustion>,
        unzapped_players: Vec<String>,
    ) -> Self {
        Self { player_id_touchdown, knockout_recoveries, heat_exhaustions, unzapped_players }
    }

    pub fn get_player_id_touchdown(&self) -> Option<&str> { self.player_id_touchdown.as_deref() }
    pub fn get_knockout_recoveries(&self) -> &[KnockoutRecovery] { &self.knockout_recoveries }
    pub fn get_heat_exhaustions(&self) -> &[HeatExhaustion] { &self.heat_exhaustions }
    pub fn get_unzapped_players(&self) -> &[String] { &self.unzapped_players }

    pub fn to_json_value(&self) -> serde_json::Value {
        let knockout_array: Vec<serde_json::Value> =
            self.knockout_recoveries.iter().map(|k| k.to_json_value()).collect();
        let heat_array: Vec<serde_json::Value> =
            self.heat_exhaustions.iter().map(|h| h.to_json_value()).collect();
        let unzap_array: Vec<serde_json::Value> =
            self.unzapped_players.iter().map(|id| serde_json::json!({ "playerId": id })).collect();
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "playerIdTouchdown": self.player_id_touchdown,
            "knockoutRecoveryArray": knockout_array,
            "heatExhaustionArray": heat_array,
            "unzapArray": unzap_array,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            player_id_touchdown: json["playerIdTouchdown"].as_str().map(str::to_string),
            knockout_recoveries: json["knockoutRecoveryArray"]
                .as_array()
                .map(|a| a.iter().map(KnockoutRecovery::from_json).collect())
                .unwrap_or_default(),
            heat_exhaustions: json["heatExhaustionArray"]
                .as_array()
                .map(|a| a.iter().map(HeatExhaustion::from_json).collect())
                .unwrap_or_default(),
            unzapped_players: json["unzapArray"]
                .as_array()
                .map(|a| {
                    a.iter()
                        .filter_map(|v| {
                            v.get("playerId").and_then(|p| p.as_str()).map(str::to_string)
                        })
                        .collect()
                })
                .unwrap_or_default(),
        }
    }
}

impl IReport for ReportTurnEnd {
    fn to_json(&self) -> Option<serde_json::Value> { Some(self.to_json_value()) }
    fn get_id(&self) -> ReportId { ReportId::TURN_END }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportTurnEnd {
        ReportTurnEnd::new(
            Some("scorer".into()),
            vec![],
            vec![HeatExhaustion::new("home_03".into(), 2)],
            vec!["away_07".into()],
        )
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportTurnEnd::from_json(&json);
        assert_eq!(restored.player_id_touchdown, original.player_id_touchdown);
        assert_eq!(restored.unzapped_players, original.unzapped_players);
        assert_eq!(restored.heat_exhaustions.len(), 1);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("turnEnd"));
    }

    /// The whole point of the BB2016 class: Java's `bb2016/ReportTurnEnd.toJsonValue` stops at
    /// UNZAP_ARRAY and never writes HEAT_ROLL.
    #[test]
    fn bb2016_turn_end_omits_heat_roll() {
        let json = make().to_json_value();
        assert!(json.get("heatRoll").is_none(), "BB2016 ReportTurnEnd must not write heatRoll");
    }
}
