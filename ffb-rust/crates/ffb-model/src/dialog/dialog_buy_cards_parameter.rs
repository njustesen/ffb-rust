use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogBuyCardsParameter.
/// Note: CardType serialized as String name (CardType is a stub).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogBuyCardsParameter {
    pub team_id: Option<String>,
    pub available_gold: i32,
    pub available_cards: i32,
    pub nr_of_cards_per_type: HashMap<String, i32>,
}

impl DialogBuyCardsParameter {
    pub fn get_team_id(&self) -> Option<&str> { self.team_id.as_deref() }
    pub fn get_available_gold(&self) -> i32 { self.available_gold }
    pub fn get_available_cards(&self) -> i32 { self.available_cards }
    pub fn get_nr_of_cards_per_type(&self) -> &HashMap<String, i32> { &self.nr_of_cards_per_type }
}

impl IDialogParameter for DialogBuyCardsParameter {
    fn get_id(&self) -> DialogId { DialogId::BUY_CARDS }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dialog_id_is_buy_cards() {
        assert_eq!(DialogBuyCardsParameter::default().get_id(), DialogId::BUY_CARDS);
    }

    #[test]
    fn stores_gold_and_card_counts() {
        let p = DialogBuyCardsParameter {
            team_id: Some("t1".into()),
            available_gold: 100_000,
            available_cards: 3,
            ..Default::default()
        };
        assert_eq!(p.get_available_gold(), 100_000);
        assert_eq!(p.get_available_cards(), 3);
        assert_eq!(p.get_team_id(), Some("t1"));
    }
}
