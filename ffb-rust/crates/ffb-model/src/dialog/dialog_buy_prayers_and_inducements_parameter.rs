use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogBuyPrayersAndInducementsParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogBuyPrayersAndInducementsParameter {
    pub team_id: Option<String>,
    pub available_gold: i32,
    pub petty_cash: i32,
    pub treasury: i32,
    pub uses_treasury: bool,
}

impl DialogBuyPrayersAndInducementsParameter {
    pub fn get_team_id(&self) -> Option<&str> { self.team_id.as_deref() }
    pub fn get_available_gold(&self) -> i32 { self.available_gold }
    pub fn get_petty_cash(&self) -> i32 { self.petty_cash }
    pub fn get_treasury(&self) -> i32 { self.treasury }
    pub fn is_uses_treasury(&self) -> bool { self.uses_treasury }
}

impl IDialogParameter for DialogBuyPrayersAndInducementsParameter {
    fn get_id(&self) -> DialogId { DialogId::BUY_PRAYERS_AND_INDUCEMENTS }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dialog_id_is_buy_prayers_and_inducements() {
        assert_eq!(DialogBuyPrayersAndInducementsParameter::default().get_id(), DialogId::BUY_PRAYERS_AND_INDUCEMENTS);
    }
    #[test]
    fn stores_gold_and_treasury() {
        let p = DialogBuyPrayersAndInducementsParameter { available_gold: 100_000, treasury: 50_000, ..Default::default() };
        assert_eq!(p.get_available_gold(), 100_000);
        assert_eq!(p.get_treasury(), 50_000);
    }
}
