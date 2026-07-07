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
}
