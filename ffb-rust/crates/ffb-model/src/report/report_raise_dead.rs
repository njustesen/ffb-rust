use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportRaiseDead.java`.
#[derive(Debug, Clone)]
pub struct ReportRaiseDead {
    pub player_id: String,
    pub position: Option<String>,
    pub nurgles_rot: bool,
}

impl ReportRaiseDead {
    pub fn new(player_id: String, position: Option<String>, nurgles_rot: bool) -> Self {
        Self { player_id, position, nurgles_rot }
    }

    pub fn get_player_id(&self) -> &str { &self.player_id }
    pub fn get_position(&self) -> Option<&str> { self.position.as_deref() }
    pub fn is_nurgles_rot(&self) -> bool { self.nurgles_rot }

    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "playerId": self.player_id,
            "nurglesRot": self.nurgles_rot,
            "positionName": self.position,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            player_id: json["playerId"].as_str().unwrap_or("").to_string(),
            position: json["positionName"].as_str().map(str::to_string),
            nurgles_rot: json["nurglesRot"].as_bool().unwrap_or(false),
        }
    }
}

impl IReport for ReportRaiseDead {
    fn get_id(&self) -> ReportId { ReportId::RAISE_DEAD }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportRaiseDead {
        ReportRaiseDead::new("p1".into(), Some("Zombie".into()), true)
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::RAISE_DEAD);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "raiseDead");
    }

    #[test]
    fn fields() {
        let r = make();
        assert_eq!(r.get_player_id(), "p1");
        assert_eq!(r.get_position(), Some("Zombie"));
        assert!(r.is_nurgles_rot());
    }

    #[test]
    fn no_position() {
        let r = ReportRaiseDead::new("p2".into(), None, false);
        assert_eq!(r.get_position(), None);
    }

    #[test]
    fn nurgles_rot_false() {
        let r = ReportRaiseDead::new("p3".into(), Some("Ghoul".into()), false);
        assert!(!r.is_nurgles_rot());
        assert_eq!(r.get_player_id(), "p3");
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportRaiseDead::from_json(&json);
        assert_eq!(restored.player_id, original.player_id);
        assert_eq!(restored.position, original.position);
        assert_eq!(restored.nurgles_rot, original.nurgles_rot);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("raiseDead"));
    }
}
