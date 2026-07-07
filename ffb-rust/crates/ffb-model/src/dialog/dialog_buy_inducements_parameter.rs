use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogBuyInducementsParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogBuyInducementsParameter {
    pub team_id: Option<String>,
    pub available_gold: i32,
}

impl DialogBuyInducementsParameter {
    pub fn get_team_id(&self) -> Option<&str> { self.team_id.as_deref() }
    pub fn get_available_gold(&self) -> i32 { self.available_gold }
}

impl IDialogParameter for DialogBuyInducementsParameter {
    fn get_id(&self) -> DialogId { DialogId::BUY_INDUCEMENTS }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dialog_id_is_buy_inducements() {
        assert_eq!(DialogBuyInducementsParameter::default().get_id(), DialogId::BUY_INDUCEMENTS);
    }

    #[test]
    fn stores_team_id_and_gold() {
        let p = DialogBuyInducementsParameter { team_id: Some("team1".into()), available_gold: 150_000 };
        assert_eq!(p.get_team_id(), Some("team1"));
        assert_eq!(p.get_available_gold(), 150_000);
    }

    #[test]
    fn default_is_sensible() {
        let p = DialogBuyInducementsParameter::default();
        assert!(p.get_team_id().is_none());
        assert_eq!(p.get_available_gold(), 0);
    }

    #[test]
    fn serde_round_trip() {
        let p = DialogBuyInducementsParameter { team_id: Some("teamY".into()), available_gold: 200_000 };
        let json = serde_json::to_string(&p).unwrap();
        let back: DialogBuyInducementsParameter = serde_json::from_str(&json).unwrap();
        assert_eq!(back.get_team_id(), Some("teamY"));
        assert_eq!(back.get_available_gold(), 200_000);
    }

    #[test]
    fn zero_gold_is_edge_case() {
        let p = DialogBuyInducementsParameter { team_id: None, available_gold: 0 };
        assert!(p.get_team_id().is_none());
        assert_eq!(p.get_available_gold(), 0);
    }
}
