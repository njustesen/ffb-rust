use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportPenaltyShootout.java`.
#[derive(Debug, Clone)]
pub struct ReportPenaltyShootout {
    pub roll_home: i32,
    pub roll_away: i32,
    pub score_home: i32,
    pub score_away: i32,
    pub roll_count: Option<String>,
    pub winning_team: Option<String>,
    pub home_team_won_penalty: Option<bool>,
}

impl ReportPenaltyShootout {
    pub fn new(
        roll_home: i32,
        score_home: i32,
        roll_away: i32,
        score_away: i32,
        home_team_won_penalty: Option<bool>,
        roll_count: Option<String>,
        winning_team: Option<String>,
    ) -> Self {
        Self { roll_home, roll_away, score_home, score_away, roll_count, winning_team, home_team_won_penalty }
    }

    pub fn get_roll_home(&self) -> i32 { self.roll_home }
    pub fn get_roll_away(&self) -> i32 { self.roll_away }
    pub fn get_score_home(&self) -> i32 { self.score_home }
    pub fn get_score_away(&self) -> i32 { self.score_away }
    pub fn get_roll_count(&self) -> Option<&str> { self.roll_count.as_deref() }
    pub fn get_winning_team(&self) -> Option<&str> { self.winning_team.as_deref() }
    pub fn get_home_team_won_penalty(&self) -> Option<bool> { self.home_team_won_penalty }
}

impl IReport for ReportPenaltyShootout {
    fn get_id(&self) -> ReportId { ReportId::PENALTY_SHOOTOUT }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportPenaltyShootout {
        ReportPenaltyShootout::new(4, 1, 3, 0, Some(true), Some("1".into()), Some("home".into()))
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::PENALTY_SHOOTOUT); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "penaltyShootout"); }

    #[test]
    fn get_roll_home() { assert_eq!(make().get_roll_home(), 4); }

    #[test]
    fn get_home_team_won_penalty() { assert_eq!(make().get_home_team_won_penalty(), Some(true)); }
    #[test]
    fn get_name_is_nonempty() {
        assert!(!make().get_name().is_empty());
    }
}
