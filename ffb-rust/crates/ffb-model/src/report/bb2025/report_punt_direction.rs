use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;
use crate::enums::Direction;

/// 1:1 translation of `ReportPuntDirection.java`.
#[derive(Debug, Clone)]
pub struct ReportPuntDirection {
    pub direction: Option<Direction>,
    pub direction_roll: i32,
    pub player_id: String,
    pub out_of_bounds: bool,
}

impl ReportPuntDirection {
    pub fn new(direction: Option<Direction>, direction_roll: i32, player_id: String, out_of_bounds: bool) -> Self {
        Self { direction, direction_roll, player_id, out_of_bounds }
    }

    pub fn get_direction(&self) -> Option<Direction> { self.direction }
    pub fn get_direction_roll(&self) -> i32 { self.direction_roll }
    pub fn get_player_id(&self) -> &str { &self.player_id }
    pub fn is_out_of_bounds(&self) -> bool { self.out_of_bounds }

    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "direction": self.direction.map(|d| d.name()),
            "directionRoll": self.direction_roll,
            "playerId": self.player_id,
            "outOfBounds": self.out_of_bounds,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            direction: json["direction"].as_str().and_then(Direction::from_name),
            direction_roll: json["directionRoll"].as_i64().unwrap_or(0) as i32,
            player_id: json["playerId"].as_str().unwrap_or("").to_string(),
            out_of_bounds: json["outOfBounds"].as_bool().unwrap_or(false),
        }
    }
}

impl IReport for ReportPuntDirection {
    fn get_id(&self) -> ReportId { ReportId::PUNT_DIRECTION_ROLL }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportPuntDirection {
        ReportPuntDirection::new(Some(Direction::North), 3, "p1".into(), false)
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::PUNT_DIRECTION_ROLL);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "puntDirectionRoll");
    }

    #[test]
    fn fields() {
        let r = make();
        assert_eq!(r.get_direction(), Some(Direction::North));
        assert_eq!(r.get_direction_roll(), 3);
        assert!(!r.is_out_of_bounds());
    }

    #[test]
    fn player_id() {
        let r = make();
        assert_eq!(r.get_player_id(), "p1");
    }

    #[test]
    fn out_of_bounds_and_no_direction() {
        let r = ReportPuntDirection::new(None, 7, "p2".into(), true);
        assert_eq!(r.get_direction(), None);
        assert!(r.is_out_of_bounds());
        assert_eq!(r.get_player_id(), "p2");
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportPuntDirection::from_json(&json);
        assert_eq!(restored.direction, original.direction);
        assert_eq!(restored.direction_roll, original.direction_roll);
        assert_eq!(restored.player_id, original.player_id);
        assert_eq!(restored.out_of_bounds, original.out_of_bounds);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("puntDirectionRoll"));
    }
}
