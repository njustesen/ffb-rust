use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportTrapDoor.java`.
#[derive(Debug, Clone)]
pub struct ReportTrapDoor {
    pub player_id: Option<String>,
    pub escaped: bool,
    pub roll: i32,
}

impl ReportTrapDoor {
    pub fn new(player_id: Option<String>, roll: i32, escaped: bool) -> Self {
        Self { player_id, escaped, roll }
    }

    pub fn get_player_id(&self) -> Option<&str> { self.player_id.as_deref() }
    pub fn is_escaped(&self) -> bool { self.escaped }
    pub fn get_roll(&self) -> i32 { self.roll }

    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "escaped": self.escaped,
            "playerId": self.player_id,
            "roll": self.roll,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            player_id: json["playerId"].as_str().map(str::to_string),
            escaped: json["escaped"].as_bool().unwrap_or(false),
            roll: json["roll"].as_i64().unwrap_or(0) as i32,
        }
    }
}

impl IReport for ReportTrapDoor {
    fn get_id(&self) -> ReportId { ReportId::TRAP_DOOR }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportTrapDoor {
        ReportTrapDoor::new(Some("p1".into()), 4, true)
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::TRAP_DOOR); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "trapDoor"); }

    #[test]
    fn get_roll() { assert_eq!(make().get_roll(), 4); }

    #[test]
    fn is_escaped() { assert!(make().is_escaped()); }
    #[test]
    fn get_name_is_nonempty() {
        assert!(!make().get_name().is_empty());
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportTrapDoor::from_json(&json);
        assert_eq!(restored.player_id, original.player_id);
        assert_eq!(restored.escaped, original.escaped);
        assert_eq!(restored.roll, original.roll);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("trapDoor"));
    }
}
