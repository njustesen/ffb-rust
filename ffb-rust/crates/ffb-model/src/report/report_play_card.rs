use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportPlayCard.java`.
/// `Card` is represented as a `String` (card name) to avoid a dependency cycle.
#[derive(Debug, Clone)]
pub struct ReportPlayCard {
    pub team_id: String,
    pub card: String,
    pub player_id: Option<String>,
}

impl ReportPlayCard {
    pub fn new(team_id: String, card: String) -> Self {
        Self { team_id, card, player_id: None }
    }

    pub fn new_with_player(team_id: String, card: String, player_id: Option<String>) -> Self {
        Self { team_id, card, player_id }
    }

    pub fn get_team_id(&self) -> &str { &self.team_id }
    pub fn get_card(&self) -> &str { &self.card }
    pub fn get_player_id(&self) -> Option<&str> { self.player_id.as_deref() }

    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "teamId": self.team_id,
            "card": self.card,
            "playerId": self.player_id,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            team_id: json["teamId"].as_str().unwrap_or("").to_string(),
            card: json["card"].as_str().unwrap_or("").to_string(),
            player_id: json["playerId"].as_str().map(str::to_string),
        }
    }
}

impl IReport for ReportPlayCard {
    fn get_id(&self) -> ReportId { ReportId::PLAY_CARD }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportPlayCard {
        ReportPlayCard::new_with_player("team1".into(), "BlockCard".into(), Some("p1".into()))
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::PLAY_CARD);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "playCard");
    }

    #[test]
    fn fields() {
        let r = make();
        assert_eq!(r.get_team_id(), "team1");
        assert_eq!(r.get_card(), "BlockCard");
        assert_eq!(r.get_player_id(), Some("p1"));
    }

    #[test]
    fn new_without_player() {
        let r = ReportPlayCard::new("team2".into(), "PoisonDagger".into());
        assert_eq!(r.get_team_id(), "team2");
        assert_eq!(r.get_card(), "PoisonDagger");
        assert_eq!(r.get_player_id(), None);
    }

    #[test]
    fn no_player_id() {
        let r = ReportPlayCard::new_with_player("team1".into(), "SomeCard".into(), None);
        assert_eq!(r.get_player_id(), None);
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportPlayCard::from_json(&json);
        assert_eq!(restored.team_id, original.team_id);
        assert_eq!(restored.card, original.card);
        assert_eq!(restored.player_id, original.player_id);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("playCard"));
    }
}
