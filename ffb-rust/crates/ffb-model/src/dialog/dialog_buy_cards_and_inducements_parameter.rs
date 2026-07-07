use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogBuyCardsAndInducementsParameter.
/// Note: CardType/CardChoices serialized as String (stubs not yet translated).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogBuyCardsAndInducementsParameter {
    pub team_id: Option<String>,
    pub available_gold: i32,
    pub card_slots: i32,
    pub card_price: i32,
    pub nr_of_cards_per_type: HashMap<String, i32>,
    /// CardChoices serialized by name (CardChoices stub not yet translated).
    pub card_choices: Option<String>,
    pub can_buy_cards: bool,
    pub uses_treasury: bool,
}

impl DialogBuyCardsAndInducementsParameter {
    pub fn get_team_id(&self) -> Option<&str> { self.team_id.as_deref() }
    pub fn get_available_gold(&self) -> i32 { self.available_gold }
    pub fn get_card_slots(&self) -> i32 { self.card_slots }
    pub fn get_card_price(&self) -> i32 { self.card_price }
    pub fn can_buy_cards(&self) -> bool { self.can_buy_cards }
    pub fn uses_treasury(&self) -> bool { self.uses_treasury }
}

impl IDialogParameter for DialogBuyCardsAndInducementsParameter {
    fn get_id(&self) -> DialogId { DialogId::BUY_CARDS_AND_INDUCEMENTS }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dialog_id_is_buy_cards_and_inducements() {
        assert_eq!(DialogBuyCardsAndInducementsParameter::default().get_id(), DialogId::BUY_CARDS_AND_INDUCEMENTS);
    }
    #[test]
    fn stores_gold_and_card_fields() {
        let p = DialogBuyCardsAndInducementsParameter { available_gold: 80_000, card_slots: 2, can_buy_cards: true, ..Default::default() };
        assert_eq!(p.get_available_gold(), 80_000);
        assert_eq!(p.get_card_slots(), 2);
        assert!(p.can_buy_cards());
    }

    #[test]
    fn default_is_sensible() {
        let p = DialogBuyCardsAndInducementsParameter::default();
        assert!(p.get_team_id().is_none());
        assert_eq!(p.get_available_gold(), 0);
        assert_eq!(p.get_card_slots(), 0);
        assert_eq!(p.get_card_price(), 0);
        assert!(!p.can_buy_cards());
        assert!(!p.uses_treasury());
    }

    #[test]
    fn accessor_methods_with_non_default_values() {
        let p = DialogBuyCardsAndInducementsParameter {
            team_id: Some("home".into()),
            available_gold: 150_000,
            card_slots: 3,
            card_price: 10_000,
            can_buy_cards: true,
            uses_treasury: true,
            ..Default::default()
        };
        assert_eq!(p.get_team_id(), Some("home"));
        assert_eq!(p.get_available_gold(), 150_000);
        assert_eq!(p.get_card_slots(), 3);
        assert_eq!(p.get_card_price(), 10_000);
        assert!(p.can_buy_cards());
        assert!(p.uses_treasury());
    }

    #[test]
    fn zero_gold_is_edge_case() {
        let p = DialogBuyCardsAndInducementsParameter {
            available_gold: 0,
            can_buy_cards: false,
            ..Default::default()
        };
        assert_eq!(p.get_available_gold(), 0);
        assert!(!p.can_buy_cards());
    }
}
