use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportNoPlayersToField.java`.
#[derive(Debug, Clone)]
pub struct ReportNoPlayersToField {
    pub team_id: String,
}

impl ReportNoPlayersToField {
    pub fn new(team_id: String) -> Self {
        Self { team_id }
    }

    pub fn get_team_id(&self) -> &str { &self.team_id }
}

impl IReport for ReportNoPlayersToField {
    fn get_id(&self) -> ReportId { ReportId::NO_PLAYERS_TO_FIELD }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportNoPlayersToField {
        ReportNoPlayersToField::new("team1".into())
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::NO_PLAYERS_TO_FIELD);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "noPlayersToField");
    }

    #[test]
    fn fields() {
        assert_eq!(make().get_team_id(), "team1");
    }

    #[test]
    fn away_team_id() {
        let r = ReportNoPlayersToField::new("away_team".into());
        assert_eq!(r.get_team_id(), "away_team");
    }

    #[test]
    fn empty_team_id() {
        let r = ReportNoPlayersToField::new("".into());
        assert_eq!(r.get_team_id(), "");
    }
}
