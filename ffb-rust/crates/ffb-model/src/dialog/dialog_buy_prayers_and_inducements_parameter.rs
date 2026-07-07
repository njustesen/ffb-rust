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

    #[test]
    fn default_is_sensible() {
        let p = DialogBuyPrayersAndInducementsParameter::default();
        assert!(p.get_team_id().is_none());
        assert_eq!(p.get_available_gold(), 0);
        assert_eq!(p.get_petty_cash(), 0);
        assert_eq!(p.get_treasury(), 0);
        assert!(!p.is_uses_treasury());
    }

    #[test]
    fn accessor_methods_with_non_default_values() {
        let p = DialogBuyPrayersAndInducementsParameter {
            team_id: Some("home".into()),
            available_gold: 120_000,
            petty_cash: 30_000,
            treasury: 90_000,
            uses_treasury: true,
        };
        assert_eq!(p.get_team_id(), Some("home"));
        assert_eq!(p.get_available_gold(), 120_000);
        assert_eq!(p.get_petty_cash(), 30_000);
        assert_eq!(p.get_treasury(), 90_000);
        assert!(p.is_uses_treasury());
    }

    #[test]
    fn zero_petty_cash_is_edge_case() {
        let p = DialogBuyPrayersAndInducementsParameter {
            petty_cash: 0,
            uses_treasury: false,
            ..Default::default()
        };
        assert_eq!(p.get_petty_cash(), 0);
        assert!(!p.is_uses_treasury());
    }
}
