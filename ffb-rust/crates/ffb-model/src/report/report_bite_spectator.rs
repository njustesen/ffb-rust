use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportBiteSpectator.java`.
#[derive(Debug, Clone)]
pub struct ReportBiteSpectator {
    pub player_id: String,
}

impl ReportBiteSpectator {
    pub fn new(player_id: String) -> Self {
        Self { player_id }
    }

    pub fn get_player_id(&self) -> &str { &self.player_id }
}

impl IReport for ReportBiteSpectator {
    fn get_id(&self) -> ReportId { ReportId::BITE_SPECTATOR }
}

impl ReportBiteSpectator {
    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "playerId": self.player_id,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            player_id: json["playerId"].as_str().unwrap_or("").to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportBiteSpectator {
        ReportBiteSpectator::new("p1".into())
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::BITE_SPECTATOR);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "biteSpectator");
    }

    #[test]
    fn get_player_id() {
        assert_eq!(make().get_player_id(), "p1");
    }

    #[test]
    fn different_player_id() {
        let r = ReportBiteSpectator::new("p99".into());
        assert_eq!(r.get_player_id(), "p99");
    }

    #[test]
    fn player_id_matches_field() {
        let r = ReportBiteSpectator::new("spectator_biter".into());
        assert_eq!(r.get_player_id(), r.player_id.as_str());
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportBiteSpectator::from_json(&json);
        assert_eq!(restored.player_id, original.player_id);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("biteSpectator"));
    }
}
