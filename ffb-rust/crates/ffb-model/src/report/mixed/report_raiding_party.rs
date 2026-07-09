use crate::enums::Direction;
use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportRaidingParty.java`.
#[derive(Debug, Clone)]
pub struct ReportRaidingParty {
    pub player_id: Option<String>,
    pub other_player_id: Option<String>,
    pub direction: Option<Direction>,
}

impl ReportRaidingParty {
    pub fn new(
        player_id: Option<String>,
        other_player_id: Option<String>,
        direction: Option<Direction>,
    ) -> Self {
        Self { player_id, other_player_id, direction }
    }

    pub fn get_player_id(&self) -> Option<&str> { self.player_id.as_deref() }
    pub fn get_other_player_id(&self) -> Option<&str> { self.other_player_id.as_deref() }
    pub fn get_direction(&self) -> Option<Direction> { self.direction }
}

impl IReport for ReportRaidingParty {
    fn get_id(&self) -> ReportId { ReportId::RAIDING_PARTY }
}

impl ReportRaidingParty {
    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "playerId": self.player_id,
            "playerIdOtherPlayer": self.other_player_id,
            "direction": self.direction.map(|d| d.name()),
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            player_id: json["playerId"].as_str().map(str::to_string),
            other_player_id: json["playerIdOtherPlayer"].as_str().map(str::to_string),
            direction: json["direction"].as_str().and_then(Direction::from_name),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportRaidingParty {
        ReportRaidingParty::new(Some("p1".into()), Some("p2".into()), Some(Direction::Northeast))
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::RAIDING_PARTY); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "raidingParty"); }

    #[test]
    fn get_other_player_id() { assert_eq!(make().get_other_player_id(), Some("p2")); }

    #[test]
    fn get_direction() { assert_eq!(make().get_direction(), Some(Direction::Northeast)); }
    #[test]
    fn get_name_is_nonempty() {
        assert!(!make().get_name().is_empty());
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportRaidingParty::from_json(&json);
        assert_eq!(restored.player_id, original.player_id);
        assert_eq!(restored.other_player_id, original.other_player_id);
        assert_eq!(restored.direction, original.direction);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("raidingParty"));
    }
}
