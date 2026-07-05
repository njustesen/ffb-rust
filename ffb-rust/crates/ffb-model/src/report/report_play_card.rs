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
}
