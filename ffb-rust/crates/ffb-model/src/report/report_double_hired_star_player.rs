use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportDoubleHiredStarPlayer.java`.
#[derive(Debug, Clone)]
pub struct ReportDoubleHiredStarPlayer {
    /// Translated from `fStarPlayerName`.
    pub star_player_name: String,
}

impl ReportDoubleHiredStarPlayer {
    pub fn new(star_player_name: String) -> Self {
        Self { star_player_name }
    }

    pub fn get_star_player_name(&self) -> &str {
        &self.star_player_name
    }
}

impl IReport for ReportDoubleHiredStarPlayer {
    fn get_id(&self) -> ReportId {
        ReportId::DOUBLE_HIRED_STAR_PLAYER
    }
}

impl ReportDoubleHiredStarPlayer {
    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "starPlayerName": self.star_player_name,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            star_player_name: json["starPlayerName"].as_str().unwrap_or("").to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportDoubleHiredStarPlayer {
        ReportDoubleHiredStarPlayer::new("Griff Oberwald".into())
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::DOUBLE_HIRED_STAR_PLAYER);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "doubleHiredStarPlayer");
    }

    #[test]
    fn star_player_name_getter() {
        assert_eq!(make().get_star_player_name(), "Griff Oberwald");
    }

    #[test]
    fn different_star_player() {
        let r = ReportDoubleHiredStarPlayer::new("Morg 'n' Thorg".into());
        assert_eq!(r.get_star_player_name(), "Morg 'n' Thorg");
    }

    #[test]
    fn star_player_name_matches_field() {
        let r = make();
        assert_eq!(r.get_star_player_name(), r.star_player_name.as_str());
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportDoubleHiredStarPlayer::from_json(&json);
        assert_eq!(restored.star_player_name, original.star_player_name);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("doubleHiredStarPlayer"));
    }
}
