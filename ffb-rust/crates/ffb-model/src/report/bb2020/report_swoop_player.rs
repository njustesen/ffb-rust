use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;
use crate::types::FieldCoordinate;
use crate::enums::Direction;

/// 1:1 translation of `ReportSwoopPlayer.java` (bb2020).
#[derive(Debug, Clone)]
pub struct ReportSwoopPlayer {
    pub start_coordinate: FieldCoordinate,
    pub end_coordinate: FieldCoordinate,
    pub direction: Direction,
    pub distance: i32,
}

impl ReportSwoopPlayer {
    pub fn new(
        start_coordinate: FieldCoordinate,
        end_coordinate: FieldCoordinate,
        direction: Direction,
        distance: i32,
    ) -> Self {
        Self { start_coordinate, end_coordinate, direction, distance }
    }

    pub fn get_start_coordinate(&self) -> &FieldCoordinate { &self.start_coordinate }
    pub fn get_end_coordinate(&self) -> &FieldCoordinate { &self.end_coordinate }
    pub fn get_direction(&self) -> Direction { self.direction }
    pub fn get_distance(&self) -> i32 { self.distance }

    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "startCoordinate": {"x": self.start_coordinate.x, "y": self.start_coordinate.y},
            "endCoordinate": {"x": self.end_coordinate.x, "y": self.end_coordinate.y},
            "distance": self.distance,
            "scatterDirection": self.direction.name(),
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            start_coordinate: FieldCoordinate::new(
                json["startCoordinate"]["x"].as_i64().unwrap_or(0) as i32,
                json["startCoordinate"]["y"].as_i64().unwrap_or(0) as i32,
            ),
            end_coordinate: FieldCoordinate::new(
                json["endCoordinate"]["x"].as_i64().unwrap_or(0) as i32,
                json["endCoordinate"]["y"].as_i64().unwrap_or(0) as i32,
            ),
            direction: json["scatterDirection"]
                .as_str()
                .and_then(Direction::from_name)
                .unwrap_or(Direction::North),
            distance: json["distance"].as_i64().unwrap_or(0) as i32,
        }
    }
}

impl IReport for ReportSwoopPlayer {
    fn get_id(&self) -> ReportId { ReportId::SWOOP_PLAYER }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportSwoopPlayer {
        ReportSwoopPlayer::new(
            FieldCoordinate::new(5, 7),
            FieldCoordinate::new(8, 7),
            Direction::East,
            3,
        )
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportSwoopPlayer::from_json(&json);
        assert_eq!(restored.start_coordinate.x, original.start_coordinate.x);
        assert_eq!(restored.start_coordinate.y, original.start_coordinate.y);
        assert_eq!(restored.end_coordinate.x, original.end_coordinate.x);
        assert_eq!(restored.end_coordinate.y, original.end_coordinate.y);
        assert_eq!(restored.direction, original.direction);
        assert_eq!(restored.distance, original.distance);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("swoopPlayer"));
    }
}
