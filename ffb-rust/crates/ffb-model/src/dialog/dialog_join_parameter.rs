use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogJoinParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogJoinParameter;

impl IDialogParameter for DialogJoinParameter {
    fn get_id(&self) -> DialogId { DialogId::JOIN }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn serde_round_trip() {
        let p = DialogJoinParameter;
        let json = serde_json::to_string(&p).unwrap();
        let back: DialogJoinParameter = serde_json::from_str(&json).unwrap();
        assert_eq!(back.get_id(), DialogId::JOIN);
    }

}
