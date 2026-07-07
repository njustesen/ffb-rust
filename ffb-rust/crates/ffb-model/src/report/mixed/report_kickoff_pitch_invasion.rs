use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportKickoffPitchInvasion.java`.
#[derive(Debug, Clone)]
pub struct ReportKickoffPitchInvasion {
    pub roll_home: i32,
    pub roll_away: i32,
    pub amount: i32,
    pub affected_players: Vec<String>,
}

impl ReportKickoffPitchInvasion {
    pub fn new(
        roll_home: i32,
        roll_away: i32,
        amount: i32,
        affected_players: Vec<String>,
    ) -> Self {
        Self { roll_home, roll_away, amount, affected_players }
    }

    pub fn get_roll_home(&self) -> i32 { self.roll_home }
    pub fn get_roll_away(&self) -> i32 { self.roll_away }
    pub fn get_amount(&self) -> i32 { self.amount }
    pub fn get_affected_players(&self) -> &[String] { &self.affected_players }
}

impl IReport for ReportKickoffPitchInvasion {
    fn get_id(&self) -> ReportId { ReportId::KICKOFF_PITCH_INVASION }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportKickoffPitchInvasion {
        ReportKickoffPitchInvasion::new(3, 2, 1, vec!["p1".into()])
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::KICKOFF_PITCH_INVASION); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "kickoffPitchInvasion"); }

    #[test]
    fn get_amount() { assert_eq!(make().get_amount(), 1); }

    #[test]
    fn get_roll_home_and_away() {
        let r = make();
        assert_eq!(r.get_roll_home(), 3);
        assert_eq!(r.get_roll_away(), 2);
    }

    #[test]
    fn get_affected_players() {
        let r = make();
        assert_eq!(r.get_affected_players(), &["p1".to_string()]);
    }
}
