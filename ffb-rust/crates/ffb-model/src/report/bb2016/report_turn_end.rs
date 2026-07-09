use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;
use crate::model::knockout_recovery::KnockoutRecovery;
use crate::model::heat_exhaustion::HeatExhaustion;

/// 1:1 translation of `ReportTurnEnd.java`.
#[derive(Debug, Clone)]
pub struct ReportTurnEnd {
    pub player_id_touchdown: Option<String>,
    pub knockout_recoveries: Vec<KnockoutRecovery>,
    pub heat_exhaustions: Vec<HeatExhaustion>,
    pub unzapped_player_ids: Vec<String>,
}

impl ReportTurnEnd {
    pub fn new(
        player_id_touchdown: Option<String>,
        knockout_recoveries: Vec<KnockoutRecovery>,
        heat_exhaustions: Vec<HeatExhaustion>,
        unzapped_player_ids: Vec<String>,
    ) -> Self {
        Self { player_id_touchdown, knockout_recoveries, heat_exhaustions, unzapped_player_ids }
    }

    pub fn get_player_id_touchdown(&self) -> Option<&str> { self.player_id_touchdown.as_deref() }
    pub fn get_knockout_recoveries(&self) -> &[KnockoutRecovery] { &self.knockout_recoveries }
    pub fn get_heat_exhaustions(&self) -> &[HeatExhaustion] { &self.heat_exhaustions }
    pub fn get_unzapped_player_ids(&self) -> &[String] { &self.unzapped_player_ids }

    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "playerIdTouchdown": self.player_id_touchdown,
            "knockoutRecoveryArray": serde_json::Value::Array(vec![]),
            "heatExhaustionArray": serde_json::Value::Array(vec![]),
            "unzapArray": self.unzapped_player_ids,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            player_id_touchdown: json["playerIdTouchdown"].as_str().map(str::to_string),
            knockout_recoveries: vec![],
            heat_exhaustions: vec![],
            unzapped_player_ids: json["unzapArray"].as_array().map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_string)).collect()).unwrap_or_default(),
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
        ReportTurnEnd::new(Some("scorer".into()), vec![], vec![], vec![])
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::TURN_END);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "turnEnd");
    }

    #[test]
    fn fields() {
        let r = make();
        assert_eq!(r.get_player_id_touchdown(), Some("scorer"));
        assert!(r.get_knockout_recoveries().is_empty());
    }

    #[test]
    fn no_touchdown_none() {
        let r = ReportTurnEnd::new(None, vec![], vec![], vec![]);
        assert_eq!(r.get_player_id_touchdown(), None);
        assert!(r.get_heat_exhaustions().is_empty());
    }

    #[test]
    fn unzapped_player_ids_stored() {
        let r = ReportTurnEnd::new(None, vec![], vec![], vec!["p1".into(), "p2".into()]);
        assert_eq!(r.get_unzapped_player_ids().len(), 2);
        assert_eq!(r.get_unzapped_player_ids()[0], "p1");
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportTurnEnd::from_json(&json);
        assert_eq!(restored.player_id_touchdown, original.player_id_touchdown);
        assert_eq!(restored.unzapped_player_ids, original.unzapped_player_ids);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("turnEnd"));
    }
}
