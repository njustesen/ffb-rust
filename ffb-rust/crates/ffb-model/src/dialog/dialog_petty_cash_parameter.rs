use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogPettyCashParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogPettyCashParameter {
    pub team_id: Option<String>,
    pub treasury: i32,
    pub team_value: i32,
    pub opponent_team_value: i32,
}

impl DialogPettyCashParameter {
    pub fn get_team_id(&self) -> Option<&str> { self.team_id.as_deref() }
    pub fn get_treasury(&self) -> i32 { self.treasury }
    pub fn get_team_value(&self) -> i32 { self.team_value }
    pub fn get_opponent_team_value(&self) -> i32 { self.opponent_team_value }
}

impl IDialogParameter for DialogPettyCashParameter {
    fn get_id(&self) -> DialogId { DialogId::PETTY_CASH }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dialog_id_is_petty_cash() {
        assert_eq!(DialogPettyCashParameter::default().get_id(), DialogId::PETTY_CASH);
    }

    #[test]
    fn stores_treasury_and_team_values() {
        let p = DialogPettyCashParameter { treasury: 50_000, team_value: 1_000_000, opponent_team_value: 1_100_000, team_id: None };
        assert_eq!(p.get_treasury(), 50_000);
        assert_eq!(p.get_team_value(), 1_000_000);
        assert_eq!(p.get_opponent_team_value(), 1_100_000);
    }

    #[test]
    fn default_is_sensible() {
        let p = DialogPettyCashParameter::default();
        assert!(p.get_team_id().is_none());
        assert_eq!(p.get_treasury(), 0);
        assert_eq!(p.get_team_value(), 0);
        assert_eq!(p.get_opponent_team_value(), 0);
    }

    #[test]
    fn team_id_accessor() {
        let p = DialogPettyCashParameter { team_id: Some("home_team".into()), ..Default::default() };
        assert_eq!(p.get_team_id(), Some("home_team"));
    }

    #[test]
    fn zero_values_edge_case() {
        let p = DialogPettyCashParameter { treasury: 0, team_value: 0, opponent_team_value: 0, team_id: None };
        assert_eq!(p.get_treasury(), 0);
        assert_eq!(p.get_team_id(), None);
    }
}
