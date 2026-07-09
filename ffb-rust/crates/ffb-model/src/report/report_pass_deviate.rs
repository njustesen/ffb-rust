use crate::enums::Direction;
use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;
use crate::types::FieldCoordinate;

/// 1:1 translation of `ReportPassDeviate.java`.
#[derive(Debug, Clone)]
pub struct ReportPassDeviate {
    /// Translated from `fBallCoordinateEnd`.
    pub ball_coordinate_end: FieldCoordinate,
    /// Translated from `fScatterDirection`.
    pub scatter_direction: Direction,
    /// Translated from `fRollScatterDirection`.
    pub roll_scatter_direction: i32,
    /// Translated from `fRollScatterDistance`.
    pub roll_scatter_distance: i32,
    /// Translated from `ttm` (throw-team-mate flag).
    pub ttm: bool,
}

impl ReportPassDeviate {
    pub fn new(
        ball_coordinate_end: FieldCoordinate,
        scatter_direction: Direction,
        roll_scatter_direction: i32,
        roll_scatter_distance: i32,
        ttm: bool,
    ) -> Self {
        Self {
            ball_coordinate_end,
            scatter_direction,
            roll_scatter_direction,
            roll_scatter_distance,
            ttm,
        }
    }

    pub fn get_ball_coordinate_end(&self) -> &FieldCoordinate {
        &self.ball_coordinate_end
    }

    pub fn get_scatter_direction(&self) -> Direction {
        self.scatter_direction
    }

    pub fn get_roll_scatter_direction(&self) -> i32 {
        self.roll_scatter_direction
    }

    pub fn get_roll_scatter_distance(&self) -> i32 {
        self.roll_scatter_distance
    }

    pub fn is_ttm(&self) -> bool {
        self.ttm
    }

    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "ballCoordinateEnd": [self.ball_coordinate_end.x, self.ball_coordinate_end.y],
            "scatterDirection": self.scatter_direction.name(),
            "rollScatterDirection": self.roll_scatter_direction,
            "rollScatterDistance": self.roll_scatter_distance,
            "throwTeamMate": self.ttm,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        let coord = json["ballCoordinateEnd"].as_array();
        let x = coord.and_then(|a| a.first()).and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        let y = coord.and_then(|a| a.get(1)).and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        Self {
            ball_coordinate_end: FieldCoordinate::new(x, y),
            scatter_direction: json["scatterDirection"].as_str().and_then(Direction::from_name).unwrap_or(Direction::North),
            roll_scatter_direction: json["rollScatterDirection"].as_i64().unwrap_or(0) as i32,
            roll_scatter_distance: json["rollScatterDistance"].as_i64().unwrap_or(0) as i32,
            ttm: json["throwTeamMate"].as_bool().unwrap_or(false),
        }
    }
}

impl IReport for ReportPassDeviate {
    fn get_id(&self) -> ReportId {
        ReportId::PASS_DEVIATE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportPassDeviate {
        ReportPassDeviate::new(
            FieldCoordinate::new(10, 5),
            Direction::East,
            6,
            3,
            false,
        )
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::PASS_DEVIATE);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "passDeviate");
    }

    #[test]
    fn field_getters() {
        let r = make();
        assert_eq!(r.get_scatter_direction(), Direction::East);
        assert_eq!(r.get_roll_scatter_direction(), 6);
        assert_eq!(r.get_roll_scatter_distance(), 3);
        assert!(!r.is_ttm());
    }

    #[test]
    fn ttm_flag() {
        let r = ReportPassDeviate::new(FieldCoordinate::new(0, 0), Direction::North, 1, 2, true);
        assert!(r.is_ttm());
    }

    #[test]
    fn ball_coordinate_end() {
        let r = make();
        assert_eq!(r.get_ball_coordinate_end(), &FieldCoordinate::new(10, 5));
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportPassDeviate::from_json(&json);
        assert_eq!(restored.ball_coordinate_end, original.ball_coordinate_end);
        assert_eq!(restored.scatter_direction, original.scatter_direction);
        assert_eq!(restored.roll_scatter_direction, original.roll_scatter_direction);
        assert_eq!(restored.roll_scatter_distance, original.roll_scatter_distance);
        assert_eq!(restored.ttm, original.ttm);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("passDeviate"));
    }
}
