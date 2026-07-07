use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogTouchbackParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogTouchbackParameter;

impl IDialogParameter for DialogTouchbackParameter {
    fn get_id(&self) -> DialogId { DialogId::TOUCHBACK }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dialog_id_is_touchback() {
        assert_eq!(DialogTouchbackParameter.get_id(), DialogId::TOUCHBACK);
    }
    #[test]
    fn transform_preserves_id() {
        assert_eq!(DialogTouchbackParameter.transform().get_id(), DialogId::TOUCHBACK);
    }
    #[test]
    fn default_matches_unit_struct() {
        assert_eq!(DialogTouchbackParameter::default().get_id(), DialogId::TOUCHBACK);
    }
    #[test]
    fn double_transform_preserves_id() {
        let once = DialogTouchbackParameter.transform();
        assert_eq!(once.transform().get_id(), DialogId::TOUCHBACK);
    }
    #[test]
    fn id_name_is_touchback() {
        assert_eq!(DialogTouchbackParameter.get_id().get_name(), "touchback");
    }
}
