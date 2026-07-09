use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportCardDeactivated.java`.
/// `Card` is stored as its name string (card type name).
#[derive(Debug, Clone)]
pub struct ReportCardDeactivated {
    /// Card type name (replaces `Card` object).
    pub card: String,
}

impl ReportCardDeactivated {
    pub fn new(card: String) -> Self {
        Self { card }
    }

    pub fn get_card(&self) -> &str { &self.card }
}

impl IReport for ReportCardDeactivated {
    fn get_id(&self) -> ReportId { ReportId::CARD_DEACTIVATED }
}

impl ReportCardDeactivated {
    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "card": self.card,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            card: json["card"].as_str().unwrap_or("").to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportCardDeactivated {
        ReportCardDeactivated::new("CUSTARD_PIE".into())
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::CARD_DEACTIVATED);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "cardDeactivated");
    }

    #[test]
    fn get_card() {
        assert_eq!(make().get_card(), "CUSTARD_PIE");
    }

    #[test]
    fn different_card() {
        let r = ReportCardDeactivated::new("ILLEGAL_PROCEDURE".into());
        assert_eq!(r.get_card(), "ILLEGAL_PROCEDURE");
    }

    #[test]
    fn card_matches_field() {
        let r = make();
        assert_eq!(r.get_card(), r.card.as_str());
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportCardDeactivated::from_json(&json);
        assert_eq!(restored.card, original.card);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("cardDeactivated"));
    }
}
