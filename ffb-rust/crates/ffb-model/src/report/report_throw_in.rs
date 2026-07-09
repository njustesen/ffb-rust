use crate::enums::Direction;
use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportThrowIn.java`.
#[derive(Debug, Clone)]
pub struct ReportThrowIn {
    pub direction: Direction,
    pub direction_roll: i32,
    pub distance_roll: Vec<i32>,
}

impl ReportThrowIn {
    pub fn new(direction: Direction, direction_roll: i32, distance_roll: Vec<i32>) -> Self {
        Self { direction, direction_roll, distance_roll }
    }

    pub fn get_direction(&self) -> Direction { self.direction }
    pub fn get_direction_roll(&self) -> i32 { self.direction_roll }
    pub fn get_distance_roll(&self) -> &[i32] { &self.distance_roll }
}

impl IReport for ReportThrowIn {
    fn get_id(&self) -> ReportId { ReportId::THROW_IN }
}

impl ReportThrowIn {
    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "direction": self.direction.name(),
            "directionRoll": self.direction_roll,
            "distanceRoll": self.distance_roll,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            direction: json["direction"].as_str()
                .and_then(Direction::from_name)
                .unwrap_or(Direction::North),
            direction_roll: json["directionRoll"].as_i64().unwrap_or(0) as i32,
            distance_roll: json["distanceRoll"].as_array()
                .map(|a| a.iter().map(|v| v.as_i64().unwrap_or(0) as i32).collect())
                .unwrap_or_default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportThrowIn {
        ReportThrowIn::new(Direction::North, 3, vec![2, 4])
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::THROW_IN);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "throwIn");
    }

    #[test]
    fn fields() {
        let r = make();
        assert_eq!(r.get_direction(), Direction::North);
        assert_eq!(r.get_direction_roll(), 3);
        assert_eq!(r.get_distance_roll(), &[2, 4]);
    }

    #[test]
    fn different_direction() {
        let r = ReportThrowIn::new(Direction::South, 5, vec![3]);
        assert_eq!(r.get_direction(), Direction::South);
    }

    #[test]
    fn distance_roll_length() {
        let r = ReportThrowIn::new(Direction::East, 2, vec![1, 2, 3]);
        assert_eq!(r.get_distance_roll().len(), 3);
        assert_eq!(r.get_direction_roll(), 2);
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportThrowIn::from_json(&json);
        assert_eq!(restored.direction, original.direction);
        assert_eq!(restored.direction_roll, original.direction_roll);
        assert_eq!(restored.distance_roll, original.distance_roll);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("throwIn"));
    }
}
