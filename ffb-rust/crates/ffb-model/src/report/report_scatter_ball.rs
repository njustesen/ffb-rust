use crate::enums::Direction;
use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportScatterBall.java`.
#[derive(Debug, Clone)]
pub struct ReportScatterBall {
    pub directions: Vec<Direction>,
    pub rolls: Vec<i32>,
    pub gust_of_wind: bool,
}

impl ReportScatterBall {
    pub fn new(directions: Vec<Direction>, rolls: Vec<i32>, gust_of_wind: bool) -> Self {
        Self { directions, rolls, gust_of_wind }
    }

    pub fn get_directions(&self) -> &[Direction] { &self.directions }
    pub fn get_rolls(&self) -> &[i32] { &self.rolls }
    pub fn is_gust_of_wind(&self) -> bool { self.gust_of_wind }

    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "directionArray": self.directions.iter().map(|d| d.name()).collect::<Vec<_>>(),
            "rolls": self.rolls,
            "gustOfWind": self.gust_of_wind,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            directions: json["directionArray"].as_array().map(|a| a.iter().filter_map(|v| v.as_str().and_then(Direction::from_name)).collect()).unwrap_or_default(),
            rolls: json["rolls"].as_array().map(|a| a.iter().map(|v| v.as_i64().unwrap_or(0) as i32).collect()).unwrap_or_default(),
            gust_of_wind: json["gustOfWind"].as_bool().unwrap_or(false),
        }
    }
}

impl IReport for ReportScatterBall {
    fn get_id(&self) -> ReportId { ReportId::SCATTER_BALL }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportScatterBall {
        ReportScatterBall::new(vec![Direction::North, Direction::East], vec![3, 5], false)
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::SCATTER_BALL);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "scatterBall");
    }

    #[test]
    fn fields() {
        let r = make();
        assert_eq!(r.get_directions(), &[Direction::North, Direction::East]);
        assert_eq!(r.get_rolls(), &[3, 5]);
        assert!(!r.is_gust_of_wind());
    }

    #[test]
    fn gust_of_wind_true() {
        let r = ReportScatterBall::new(vec![Direction::South], vec![2], true);
        assert!(r.is_gust_of_wind());
    }

    #[test]
    fn single_direction_and_roll() {
        let r = ReportScatterBall::new(vec![Direction::West], vec![6], false);
        assert_eq!(r.get_directions(), &[Direction::West]);
        assert_eq!(r.get_rolls(), &[6]);
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportScatterBall::from_json(&json);
        assert_eq!(restored.directions, original.directions);
        assert_eq!(restored.rolls, original.rolls);
        assert_eq!(restored.gust_of_wind, original.gust_of_wind);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("scatterBall"));
    }
}
