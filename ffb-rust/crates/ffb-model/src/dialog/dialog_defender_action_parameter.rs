use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogDefenderActionParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogDefenderActionParameter;

impl IDialogParameter for DialogDefenderActionParameter {
    fn get_id(&self) -> DialogId { DialogId::DEFENDER_ACTION }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn serde_round_trip() {
        let p = DialogDefenderActionParameter;
        let json = serde_json::to_string(&p).unwrap();
        let back: DialogDefenderActionParameter = serde_json::from_str(&json).unwrap();
        assert_eq!(back.get_id(), DialogId::DEFENDER_ACTION);
    }

}
