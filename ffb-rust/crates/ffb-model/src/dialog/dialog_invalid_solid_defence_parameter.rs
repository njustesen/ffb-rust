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

    #[test]
    fn default_is_sensible() {
        let p = DialogInvalidSolidDefenceParameter::default();
        assert!(p.get_team_id().is_none());
        assert_eq!(p.get_amount(), 0);
        assert_eq!(p.get_limit(), 0);
    }

    #[test]
    fn accessor_methods_with_non_default_values() {
        let p = DialogInvalidSolidDefenceParameter {
            team_id: Some("home".into()),
            amount: 7,
            limit: 5,
        };
        assert_eq!(p.get_team_id(), Some("home"));
        assert_eq!(p.get_amount(), 7);
        assert_eq!(p.get_limit(), 5);
    }

    #[test]
    fn amount_equals_limit_is_edge_case() {
        let p = DialogInvalidSolidDefenceParameter {
            team_id: None,
            amount: 3,
            limit: 3,
        };
        assert_eq!(p.get_amount(), p.get_limit());
        assert!(p.get_team_id().is_none());
    }
}
