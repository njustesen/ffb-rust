use crate::enums::Direction;
use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportHitAndRun.java`.
#[derive(Debug, Clone)]
pub struct ReportHitAndRun {
    pub player_id: Option<String>,
    pub direction: Option<Direction>,
}

impl ReportHitAndRun {
    pub fn new(player_id: Option<String>, direction: Option<Direction>) -> Self {
        Self { player_id, direction }
    }

    pub fn get_player_id(&self) -> Option<&str> { self.player_id.as_deref() }
    pub fn get_direction(&self) -> Option<Direction> { self.direction }
}

impl IReport for ReportHitAndRun {
    fn get_id(&self) -> ReportId { ReportId::HIT_AND_RUN }
}

impl ReportHitAndRun {
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

    fn make() -> ReportHitAndRun {
        ReportHitAndRun::new(Some("p1".into()), Some(Direction::North))
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::HIT_AND_RUN); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "hitAndRun"); }

    #[test]
    fn get_direction() { assert_eq!(make().get_direction(), Some(Direction::North)); }

    #[test]
    fn get_player_id() { assert_eq!(make().get_player_id(), Some("p1")); }

    #[test]
    fn none_direction() {
        let r = ReportHitAndRun::new(None, None);
        assert_eq!(r.get_player_id(), None);
        assert_eq!(r.get_direction(), None);
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportHitAndRun::from_json(&json);
        assert_eq!(restored.player_id, original.player_id);
        assert_eq!(restored.direction, original.direction);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("hitAndRun"));
    }
}
