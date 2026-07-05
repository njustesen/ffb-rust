use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogSelectGazeTargetParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogSelectGazeTargetParameter;

impl IDialogParameter for DialogSelectGazeTargetParameter {
    fn get_id(&self) -> DialogId { DialogId::SELECT_GAZE_TARGET }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dialog_id_is_select_gaze_target() {
        assert_eq!(DialogSelectGazeTargetParameter.get_id(), DialogId::SELECT_GAZE_TARGET);
    }
    #[test]
    fn transform_preserves_id() {
        assert_eq!(DialogSelectGazeTargetParameter.transform().get_id(), DialogId::SELECT_GAZE_TARGET);
    }
}
