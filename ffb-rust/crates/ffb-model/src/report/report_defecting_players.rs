use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportDefectingPlayers.java`.
#[derive(Debug, Clone)]
pub struct ReportDefectingPlayers {
    /// Translated from `fPlayerIds`.
    pub player_ids: Vec<String>,
    /// Translated from `fRolls`.
    pub rolls: Vec<i32>,
    /// Translated from `fDefectings`.
    pub defectings: Vec<bool>,
}

impl ReportDefectingPlayers {
    pub fn new(player_ids: Vec<String>, rolls: Vec<i32>, defectings: Vec<bool>) -> Self {
        Self { player_ids, rolls, defectings }
    }

    pub fn get_player_ids(&self) -> &[String] {
        &self.player_ids
    }

    pub fn get_rolls(&self) -> &[i32] {
        &self.rolls
    }

    pub fn get_defectings(&self) -> &[bool] {
        &self.defectings
    }
}

impl IReport for ReportDefectingPlayers {
    fn get_id(&self) -> ReportId {
        ReportId::DEFECTING_PLAYERS
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportDefectingPlayers {
        ReportDefectingPlayers::new(
            vec!["p1".into(), "p2".into()],
            vec![3, 5],
            vec![true, false],
        )
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::DEFECTING_PLAYERS);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "defectingPlayers");
    }

    #[test]
    fn field_getters() {
        let r = make();
        assert_eq!(r.get_player_ids(), &["p1", "p2"]);
        assert_eq!(r.get_rolls(), &[3, 5]);
        assert_eq!(r.get_defectings(), &[true, false]);
    }

    #[test]
    fn empty_lists() {
        let r = ReportDefectingPlayers::new(vec![], vec![], vec![]);
        assert_eq!(r.get_player_ids().len(), 0);
        assert_eq!(r.get_rolls().len(), 0);
        assert_eq!(r.get_defectings().len(), 0);
    }

    #[test]
    fn single_defecting_player() {
        let r = ReportDefectingPlayers::new(vec!["p3".into()], vec![1], vec![true]);
        assert_eq!(r.get_player_ids(), &["p3"]);
        assert_eq!(r.get_defectings(), &[true]);
    }
}
