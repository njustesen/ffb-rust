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
