use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogInvalidSolidDefenceParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogInvalidSolidDefenceParameter {
    pub team_id: Option<String>,
    pub amount: i32,
    pub limit: i32,
}

impl DialogInvalidSolidDefenceParameter {
    pub fn get_team_id(&self) -> Option<&str> { self.team_id.as_deref() }
    pub fn get_amount(&self) -> i32 { self.amount }
    pub fn get_limit(&self) -> i32 { self.limit }
}

impl IDialogParameter for DialogInvalidSolidDefenceParameter {
    fn get_id(&self) -> DialogId { DialogId::INVALID_SOLID_DEFENCE }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dialog_id_is_invalid_solid_defence() {
        assert_eq!(DialogInvalidSolidDefenceParameter::default().get_id(), DialogId::INVALID_SOLID_DEFENCE);
    }
    #[test]
    fn stores_amount_and_limit() {
        let p = DialogInvalidSolidDefenceParameter { amount: 5, limit: 3, ..Default::default() };
        assert_eq!(p.get_amount(), 5);
        assert_eq!(p.get_limit(), 3);
    }
}
