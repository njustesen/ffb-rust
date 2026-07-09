use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;
use crate::types::FieldCoordinate;
use crate::enums::Direction;

/// 1:1 translation of `ReportSwoopPlayer.java` (bb2016).
#[derive(Debug, Clone)]
pub struct ReportSwoopPlayer {
    pub start_coordinate: FieldCoordinate,
    pub end_coordinate: FieldCoordinate,
    pub directions: Vec<Direction>,
    pub rolls: Vec<i32>,
}

impl ReportSwoopPlayer {
    pub fn new(
        start_coordinate: FieldCoordinate,
        end_coordinate: FieldCoordinate,
        directions: Vec<Direction>,
        rolls: Vec<i32>,
    ) -> Self {
        Self { start_coordinate, end_coordinate, directions, rolls }
    }

    pub fn get_start_coordinate(&self) -> &FieldCoordinate { &self.start_coordinate }
    pub fn get_end_coordinate(&self) -> &FieldCoordinate { &self.end_coordinate }
    pub fn get_directions(&self) -> &[Direction] { &self.directions }
    pub fn get_rolls(&self) -> &[i32] { &self.rolls }

    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "startCoordinate": {"x": self.start_coordinate.x, "y": self.start_coordinate.y},
            "endCoordinate": {"x": self.end_coordinate.x, "y": self.end_coordinate.y},
            "directionArray": self.directions.iter().map(|d| d.name()).collect::<Vec<_>>(),
            "rolls": self.rolls,
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
            directions: json["directionArray"].as_array().map(|a| {
                a.iter().filter_map(|v| v.as_str().and_then(Direction::from_name)).collect()
            }).unwrap_or_default(),
            rolls: json["rolls"].as_array().map(|a| a.iter().map(|v| v.as_i64().unwrap_or(0) as i32).collect()).unwrap_or_default(),
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
            vec![Direction::North, Direction::East],
            vec![3, 5],
        )
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::SWOOP_PLAYER);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "swoopPlayer");
    }

    #[test]
    fn fields() {
        let r = make();
        assert_eq!(r.get_directions().len(), 2);
        assert_eq!(r.get_rolls(), &[3, 5]);
    }

    #[test]
    fn coordinates_stored() {
        let r = make();
        assert_eq!(r.get_start_coordinate().x, 5);
        assert_eq!(r.get_start_coordinate().y, 7);
        assert_eq!(r.get_end_coordinate().x, 8);
        assert_eq!(r.get_end_coordinate().y, 7);
    }

    #[test]
    fn directions_contents() {
        let r = make();
        assert_eq!(r.get_directions()[0], Direction::North);
        assert_eq!(r.get_directions()[1], Direction::East);
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportSwoopPlayer::from_json(&json);
        assert_eq!(restored.start_coordinate.x, original.start_coordinate.x);
        assert_eq!(restored.rolls, original.rolls);
        assert_eq!(restored.directions.len(), original.directions.len());
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("swoopPlayer"));
    }
}
