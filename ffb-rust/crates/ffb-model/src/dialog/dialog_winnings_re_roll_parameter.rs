use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogWinningsReRollParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogWinningsReRollParameter {
    pub team_id: Option<String>,
    pub old_roll: i32,
}

impl DialogWinningsReRollParameter {
    pub fn get_team_id(&self) -> Option<&str> { self.team_id.as_deref() }
    pub fn get_old_roll(&self) -> i32 { self.old_roll }
}

impl IDialogParameter for DialogWinningsReRollParameter {
    fn get_id(&self) -> DialogId { DialogId::WINNINGS_RE_ROLL }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dialog_id_is_winnings_re_roll() {
        assert_eq!(DialogWinningsReRollParameter::default().get_id(), DialogId::WINNINGS_RE_ROLL);
    }
    #[test]
    fn stores_old_roll() {
        let p = DialogWinningsReRollParameter { old_roll: 3, ..Default::default() };
        assert_eq!(p.get_old_roll(), 3);
    }
    #[test]
    fn default_is_sensible() {
        let p = DialogWinningsReRollParameter::default();
        assert!(p.get_team_id().is_none());
        assert_eq!(p.get_old_roll(), 0);
    }
    #[test]
    fn stores_team_id() {
        let p = DialogWinningsReRollParameter { team_id: Some("home".into()), old_roll: 5 };
        assert_eq!(p.get_team_id(), Some("home"));
        assert_eq!(p.get_old_roll(), 5);
    }
    #[test]
    fn team_id_none_when_unset() {
        let p = DialogWinningsReRollParameter { team_id: None, old_roll: 0 };
        assert!(p.get_team_id().is_none());
    }
}
