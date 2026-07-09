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

    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "teamId": self.team_id,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            team_id: json["teamId"].as_str().unwrap_or("").to_string(),
        }
    }
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

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportNoPlayersToField::from_json(&json);
        assert_eq!(restored.team_id, original.team_id);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("noPlayersToField"));
    }
}
