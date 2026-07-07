use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportMostValuablePlayers.java`.
#[derive(Debug, Clone)]
pub struct ReportMostValuablePlayers {
    /// Translated from `fPlayerIdsHome`.
    pub player_ids_home: Vec<String>,
    /// Translated from `fPlayerIdsAway`.
    pub player_ids_away: Vec<String>,
}

impl ReportMostValuablePlayers {
    pub fn new(player_ids_home: Vec<String>, player_ids_away: Vec<String>) -> Self {
        Self { player_ids_home, player_ids_away }
    }

    pub fn add_player_id_home(&mut self, player_id: String) {
        self.player_ids_home.push(player_id);
    }

    pub fn add_player_id_away(&mut self, player_id: String) {
        self.player_ids_away.push(player_id);
    }

    pub fn get_player_ids_home(&self) -> &[String] {
        &self.player_ids_home
    }

    pub fn get_player_ids_away(&self) -> &[String] {
        &self.player_ids_away
    }
}

impl IReport for ReportMostValuablePlayers {
    fn get_id(&self) -> ReportId {
        ReportId::MOST_VALUABLE_PLAYERS
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportMostValuablePlayers {
        ReportMostValuablePlayers::new(
            vec!["h1".into(), "h2".into()],
            vec!["a1".into()],
        )
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::MOST_VALUABLE_PLAYERS);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "mostValuablePlayers");
    }

    #[test]
    fn field_getters() {
        let r = make();
        assert_eq!(r.get_player_ids_home(), &["h1", "h2"]);
        assert_eq!(r.get_player_ids_away(), &["a1"]);
    }

    #[test]
    fn add_player_ids() {
        let mut r = ReportMostValuablePlayers::new(vec![], vec![]);
        r.add_player_id_home("h3".into());
        r.add_player_id_away("a2".into());
        assert_eq!(r.get_player_ids_home(), &["h3"]);
        assert_eq!(r.get_player_ids_away(), &["a2"]);
    }

    #[test]
    fn empty_lists() {
        let r = ReportMostValuablePlayers::new(vec![], vec![]);
        assert!(r.get_player_ids_home().is_empty());
        assert!(r.get_player_ids_away().is_empty());
    }
}
