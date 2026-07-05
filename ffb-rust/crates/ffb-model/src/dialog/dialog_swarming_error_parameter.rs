use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogSwarmingErrorParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogSwarmingErrorParameter {
    pub allowed: i32,
    pub actual: i32,
}

impl DialogSwarmingErrorParameter {
    pub fn get_allowed(&self) -> i32 { self.allowed }
    pub fn get_actual(&self) -> i32 { self.actual }
}

impl IDialogParameter for DialogSwarmingErrorParameter {
    fn get_id(&self) -> DialogId { DialogId::SWARMING_ERROR }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dialog_id_is_swarming_error() {
        assert_eq!(DialogSwarmingErrorParameter::default().get_id(), DialogId::SWARMING_ERROR);
    }
    #[test]
    fn stores_allowed_and_actual() {
        let p = DialogSwarmingErrorParameter { allowed: 4, actual: 6 };
        assert_eq!(p.get_allowed(), 4);
        assert_eq!(p.get_actual(), 6);
    }
}
