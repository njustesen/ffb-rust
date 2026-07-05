use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportKickoffPitchInvasion.java`.
#[derive(Debug, Clone)]
pub struct ReportKickoffPitchInvasion {
    pub rolls_home: Vec<i32>,
    pub players_affected_home: Vec<bool>,
    pub rolls_away: Vec<i32>,
    pub players_affected_away: Vec<bool>,
}

impl ReportKickoffPitchInvasion {
    pub fn new(
        rolls_home: Vec<i32>,
        players_affected_home: Vec<bool>,
        rolls_away: Vec<i32>,
        players_affected_away: Vec<bool>,
    ) -> Self {
        Self { rolls_home, players_affected_home, rolls_away, players_affected_away }
    }

    pub fn get_rolls_home(&self) -> &[i32] { &self.rolls_home }
    pub fn get_players_affected_home(&self) -> &[bool] { &self.players_affected_home }
    pub fn get_rolls_away(&self) -> &[i32] { &self.rolls_away }
    pub fn get_players_affected_away(&self) -> &[bool] { &self.players_affected_away }
}

impl IReport for ReportKickoffPitchInvasion {
    fn get_id(&self) -> ReportId { ReportId::KICKOFF_PITCH_INVASION }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportKickoffPitchInvasion {
        ReportKickoffPitchInvasion::new(vec![4, 3], vec![true, false], vec![2, 5], vec![false, true])
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::KICKOFF_PITCH_INVASION);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "kickoffPitchInvasion");
    }

    #[test]
    fn fields() {
        let r = make();
        assert_eq!(r.get_rolls_home(), &[4, 3]);
        assert_eq!(r.get_players_affected_home(), &[true, false]);
        assert_eq!(r.get_rolls_away(), &[2, 5]);
    }
}
