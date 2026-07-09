use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportCardEffectRoll.java`.
/// Both `Card` and `CardEffect` are stored as name strings.
#[derive(Debug, Clone)]
pub struct ReportCardEffectRoll {
    /// Card type name (replaces `Card` object).
    pub card: String,
    pub roll: i32,
    /// `CardEffect` name; `None` if not set.
    pub card_effect: Option<String>,
}

impl ReportCardEffectRoll {
    pub fn new(card: String, roll: i32) -> Self {
        Self { card, roll, card_effect: None }
    }

    pub fn get_card(&self) -> &str { &self.card }
    pub fn get_roll(&self) -> i32 { self.roll }
    pub fn set_card_effect(&mut self, card_effect: Option<String>) { self.card_effect = card_effect; }
    pub fn get_card_effect(&self) -> Option<&str> { self.card_effect.as_deref() }
}

impl IReport for ReportCardEffectRoll {
    fn get_id(&self) -> ReportId { ReportId::CARD_EFFECT_ROLL }
}

impl ReportCardEffectRoll {
    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "card": self.card,
            "roll": self.roll,
            "cardEffect": self.card_effect,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            card: json["card"].as_str().unwrap_or("").to_string(),
            roll: json["roll"].as_i64().unwrap_or(0) as i32,
            card_effect: json["cardEffect"].as_str().map(str::to_string),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportCardEffectRoll {
        let mut r = ReportCardEffectRoll::new("DISTRACT".into(), 3);
        r.set_card_effect(Some("STUNNED".into()));
        r
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::CARD_EFFECT_ROLL);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "cardEffectRoll");
    }

    #[test]
    fn get_roll() {
        assert_eq!(make().get_roll(), 3);
    }

    #[test]
    fn get_card_and_effect() {
        let r = make();
        assert_eq!(r.get_card(), "DISTRACT");
        assert_eq!(r.get_card_effect(), Some("STUNNED"));
    }

    #[test]
    fn no_card_effect_by_default() {
        let r = ReportCardEffectRoll::new("BRIBE".into(), 5);
        assert_eq!(r.get_card_effect(), None);
        assert_eq!(r.get_roll(), 5);
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportCardEffectRoll::from_json(&json);
        assert_eq!(restored.card, original.card);
        assert_eq!(restored.roll, original.roll);
        assert_eq!(restored.card_effect, original.card_effect);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("cardEffectRoll"));
    }
}
