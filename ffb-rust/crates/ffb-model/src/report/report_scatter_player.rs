use crate::enums::Direction;
use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;
use crate::types::FieldCoordinate;

/// 1:1 translation of `ReportScatterPlayer.java`.
#[derive(Debug, Clone)]
pub struct ReportScatterPlayer {
    pub start_coordinate: FieldCoordinate,
    pub end_coordinate: FieldCoordinate,
    pub directions: Vec<Direction>,
    pub rolls: Vec<i32>,
    pub scatter: Option<bool>,
}

impl ReportScatterPlayer {
    pub fn new(
        start_coordinate: FieldCoordinate,
        end_coordinate: FieldCoordinate,
        directions: Vec<Direction>,
        rolls: Vec<i32>,
        scatter: Option<bool>,
    ) -> Self {
        Self { start_coordinate, end_coordinate, directions, rolls, scatter }
    }

    pub fn get_start_coordinate(&self) -> &FieldCoordinate { &self.start_coordinate }
    pub fn get_end_coordinate(&self) -> &FieldCoordinate { &self.end_coordinate }
    pub fn get_directions(&self) -> &[Direction] { &self.directions }
    pub fn get_rolls(&self) -> &[i32] { &self.rolls }
    pub fn get_scatter(&self) -> Option<bool> { self.scatter }

    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "startCoordinate": { "x": self.start_coordinate.x, "y": self.start_coordinate.y },
            "endCoordinate": { "x": self.end_coordinate.x, "y": self.end_coordinate.y },
            "directionArray": self.directions.iter().map(|d| d.name()).collect::<Vec<_>>(),
            "rolls": self.rolls,
            "isScatter": self.scatter,
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
            directions: json["directionArray"].as_array().map(|a| a.iter().filter_map(|v| v.as_str().and_then(Direction::from_name)).collect()).unwrap_or_default(),
            rolls: json["rolls"].as_array().map(|a| a.iter().map(|v| v.as_i64().unwrap_or(0) as i32).collect()).unwrap_or_default(),
            scatter: json["isScatter"].as_bool(),
        }
    }
}

impl IReport for ReportScatterPlayer {
    fn get_id(&self) -> ReportId { ReportId::SCATTER_PLAYER }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportScatterPlayer {
        ReportScatterPlayer::new(
            FieldCoordinate::new(3, 5),
            FieldCoordinate::new(4, 5),
            vec![Direction::East],
            vec![3],
            Some(true),
        )
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::SCATTER_PLAYER);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "scatterPlayer");
    }

    #[test]
    fn fields() {
        let r = make();
        assert_eq!(r.get_directions(), &[Direction::East]);
        assert_eq!(r.get_rolls(), &[3]);
        assert_eq!(r.get_scatter(), Some(true));
    }

    #[test]
    fn no_scatter() {
        let r = ReportScatterPlayer::new(
            FieldCoordinate::new(0, 0),
            FieldCoordinate::new(1, 0),
            vec![Direction::West],
            vec![4],
            None,
        );
        assert_eq!(r.get_scatter(), None);
    }

    #[test]
    fn coordinates() {
        let r = make();
        assert_eq!(r.get_start_coordinate(), &FieldCoordinate::new(3, 5));
        assert_eq!(r.get_end_coordinate(), &FieldCoordinate::new(4, 5));
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportScatterPlayer::from_json(&json);
        assert_eq!(restored.start_coordinate, original.start_coordinate);
        assert_eq!(restored.end_coordinate, original.end_coordinate);
        assert_eq!(restored.directions, original.directions);
        assert_eq!(restored.rolls, original.rolls);
        assert_eq!(restored.scatter, original.scatter);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("scatterPlayer"));
    }
}
