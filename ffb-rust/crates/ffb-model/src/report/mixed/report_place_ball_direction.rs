use crate::enums::Direction;
use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportPlaceBallDirection.java`.
#[derive(Debug, Clone)]
pub struct ReportPlaceBallDirection {
    pub player_id: Option<String>,
    pub direction: Option<Direction>,
}

impl ReportPlaceBallDirection {
    pub fn new(player_id: Option<String>, direction: Option<Direction>) -> Self {
        Self { player_id, direction }
    }

    pub fn get_player_id(&self) -> Option<&str> { self.player_id.as_deref() }
    pub fn get_direction(&self) -> Option<Direction> { self.direction }
}

impl IReport for ReportPlaceBallDirection {
    fn get_id(&self) -> ReportId { ReportId::PLACE_BALL_DIRECTION }
}

impl ReportPlaceBallDirection {
    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "playerId": self.player_id,
            "direction": self.direction.map(|d| d.name()),
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            player_id: json["playerId"].as_str().map(str::to_string),
            direction: json["direction"].as_str().and_then(Direction::from_name),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportPlaceBallDirection {
        ReportPlaceBallDirection::new(Some("p1".into()), Some(Direction::North))
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::PLACE_BALL_DIRECTION); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "placedBallDirection"); }

    #[test]
    fn get_direction() { assert_eq!(make().get_direction(), Some(Direction::North)); }

    #[test]
    fn get_player_id() { assert_eq!(make().get_player_id(), Some("p1")); }

    #[test]
    fn no_player_no_direction() {
        let r = ReportPlaceBallDirection::new(None, None);
        assert!(r.get_player_id().is_none());
        assert!(r.get_direction().is_none());
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportPlaceBallDirection::from_json(&json);
        assert_eq!(restored.player_id, original.player_id);
        assert_eq!(restored.direction, original.direction);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("placedBallDirection"));
    }
}
