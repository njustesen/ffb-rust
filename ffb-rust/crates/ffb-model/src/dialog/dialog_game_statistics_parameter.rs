use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogGameStatisticsParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogGameStatisticsParameter;

impl IDialogParameter for DialogGameStatisticsParameter {
    fn get_id(&self) -> DialogId { DialogId::GAME_STATISTICS }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}
