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
}
