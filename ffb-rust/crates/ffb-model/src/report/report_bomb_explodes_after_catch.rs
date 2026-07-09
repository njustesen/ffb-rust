use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportBombExplodesAfterCatch.java`.
#[derive(Debug, Clone)]
pub struct ReportBombExplodesAfterCatch {
    pub catcher_id: String,
    pub explodes: bool,
    pub roll: i32,
}

impl ReportBombExplodesAfterCatch {
    pub fn new(catcher_id: String, explodes: bool, roll: i32) -> Self {
        Self { catcher_id, explodes, roll }
    }

    pub fn get_catcher_id(&self) -> &str { &self.catcher_id }
    pub fn explodes(&self) -> bool { self.explodes }
    pub fn get_roll(&self) -> i32 { self.roll }

    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "catcherId": self.catcher_id,
            "explodes": self.explodes,
            "roll": self.roll,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            catcher_id: json["catcherId"].as_str().unwrap_or("").to_string(),
            explodes: json["explodes"].as_bool().unwrap_or(false),
            roll: json["roll"].as_i64().unwrap_or(0) as i32,
        }
    }
}

impl IReport for ReportBombExplodesAfterCatch {
    fn get_id(&self) -> ReportId { ReportId::BOMB_EXPLODES_AFTER_CATCH }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportBombExplodesAfterCatch {
        ReportBombExplodesAfterCatch::new("p1".into(), true, 5)
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::BOMB_EXPLODES_AFTER_CATCH);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "bombExplodesAfterCatch");
    }

    #[test]
    fn get_catcher_id() {
        assert_eq!(make().get_catcher_id(), "p1");
    }

    #[test]
    fn explodes_and_roll() {
        let r = make();
        assert!(r.explodes());
        assert_eq!(r.get_roll(), 5);
    }

    #[test]
    fn does_not_explode() {
        let r = ReportBombExplodesAfterCatch::new("p2".into(), false, 3);
        assert!(!r.explodes());
        assert_eq!(r.get_catcher_id(), "p2");
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportBombExplodesAfterCatch::from_json(&json);
        assert_eq!(restored.catcher_id, original.catcher_id);
        assert_eq!(restored.explodes, original.explodes);
        assert_eq!(restored.roll, original.roll);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("bombExplodesAfterCatch"));
    }
}
