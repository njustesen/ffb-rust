use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportTwoForOne.java`.
#[derive(Debug, Clone)]
pub struct ReportTwoForOne {
    pub player_id: String,
    pub partner_id: String,
    pub used: bool,
}

impl ReportTwoForOne {
    pub fn new(player_id: String, partner_id: String, used: bool) -> Self {
        Self { player_id, partner_id, used }
    }

    pub fn get_player_id(&self) -> &str { &self.player_id }
    pub fn get_partner_id(&self) -> &str { &self.partner_id }
    pub fn is_used(&self) -> bool { self.used }

    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "playerId": self.player_id,
            "used": self.used,
            "partnerId": self.partner_id,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            player_id: json["playerId"].as_str().unwrap_or("").to_string(),
            partner_id: json["partnerId"].as_str().unwrap_or("").to_string(),
            used: json["used"].as_bool().unwrap_or(false),
        }
    }
}

impl IReport for ReportTwoForOne {
    fn get_id(&self) -> ReportId { ReportId::TWO_FOR_ONE }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportTwoForOne {
        ReportTwoForOne::new("p1".into(), "p2".into(), true)
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::TWO_FOR_ONE);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "twoForOne");
    }

    #[test]
    fn fields() {
        let r = make();
        assert_eq!(r.get_player_id(), "p1");
        assert_eq!(r.get_partner_id(), "p2");
        assert!(r.is_used());
    }

    #[test]
    fn not_used() {
        let r = ReportTwoForOne::new("p3".into(), "p4".into(), false);
        assert!(!r.is_used());
        assert_eq!(r.get_player_id(), "p3");
    }

    #[test]
    fn partner_id() {
        let r = ReportTwoForOne::new("a".into(), "b".into(), true);
        assert_eq!(r.get_partner_id(), "b");
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportTwoForOne::from_json(&json);
        assert_eq!(restored.player_id, original.player_id);
        assert_eq!(restored.partner_id, original.partner_id);
        assert_eq!(restored.used, original.used);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("twoForOne"));
    }
}
