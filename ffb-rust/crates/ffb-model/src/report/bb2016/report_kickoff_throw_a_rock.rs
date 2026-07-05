use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportKickoffThrowARock.java`.
#[derive(Debug, Clone)]
pub struct ReportKickoffThrowARock {
    pub roll_home: i32,
    pub roll_away: i32,
    pub players_hit: Vec<String>,
}

impl ReportKickoffThrowARock {
    pub fn new(roll_home: i32, roll_away: i32, players_hit: Vec<String>) -> Self {
        Self { roll_home, roll_away, players_hit }
    }

    pub fn get_roll_home(&self) -> i32 { self.roll_home }
    pub fn get_roll_away(&self) -> i32 { self.roll_away }
    pub fn get_players_hit(&self) -> &[String] { &self.players_hit }
}

impl IReport for ReportKickoffThrowARock {
    fn get_id(&self) -> ReportId { ReportId::KICKOFF_THROW_A_ROCK }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportKickoffThrowARock {
        ReportKickoffThrowARock::new(4, 2, vec!["p1".into(), "p2".into()])
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::KICKOFF_THROW_A_ROCK);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "kickoffThrowARock");
    }

    #[test]
    fn fields() {
        let r = make();
        assert_eq!(r.get_roll_home(), 4);
        assert_eq!(r.get_players_hit().len(), 2);
    }
}
