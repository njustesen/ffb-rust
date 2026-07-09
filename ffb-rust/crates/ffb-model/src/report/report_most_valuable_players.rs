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

    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "playerIdsHome": self.player_ids_home,
            "playerIdsAway": self.player_ids_away,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            player_ids_home: json["playerIdsHome"].as_array().map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_string)).collect()).unwrap_or_default(),
            player_ids_away: json["playerIdsAway"].as_array().map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_string)).collect()).unwrap_or_default(),
        }
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

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportMostValuablePlayers::from_json(&json);
        assert_eq!(restored.player_ids_home, original.player_ids_home);
        assert_eq!(restored.player_ids_away, original.player_ids_away);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("mostValuablePlayers"));
    }
}
