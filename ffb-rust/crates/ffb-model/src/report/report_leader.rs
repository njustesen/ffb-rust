use crate::enums::LeaderState;
use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportLeader.java`.
#[derive(Debug, Clone)]
pub struct ReportLeader {
    /// Translated from `fTeamId`.
    pub team_id: String,
    /// Translated from `fLeaderState`.
    pub leader_state: LeaderState,
}

impl ReportLeader {
    pub fn new(team_id: String, leader_state: LeaderState) -> Self {
        Self { team_id, leader_state }
    }

    pub fn get_team_id(&self) -> &str {
        &self.team_id
    }

    pub fn get_leader_state(&self) -> LeaderState {
        self.leader_state
    }
}

impl IReport for ReportLeader {
    fn get_id(&self) -> ReportId {
        ReportId::LEADER
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportLeader {
        ReportLeader::new("team1".into(), LeaderState::Available)
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::LEADER);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "leader");
    }

    #[test]
    fn field_getters() {
        let r = make();
        assert_eq!(r.get_team_id(), "team1");
        assert_eq!(r.get_leader_state(), LeaderState::Available);
    }

    #[test]
    fn different_team_id() {
        let r = ReportLeader::new("team2".into(), LeaderState::Available);
        assert_eq!(r.get_team_id(), "team2");
    }

    #[test]
    fn used_leader_state() {
        let r = ReportLeader::new("team1".into(), LeaderState::Used);
        assert_eq!(r.get_leader_state(), LeaderState::Used);
    }
}
