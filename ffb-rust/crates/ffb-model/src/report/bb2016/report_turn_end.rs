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
}
