use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogSelectWeatherParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogSelectWeatherParameter {
    pub options: HashMap<String, i32>,
}

impl DialogSelectWeatherParameter {
    pub fn get_options(&self) -> &HashMap<String, i32> { &self.options }
}

impl IDialogParameter for DialogSelectWeatherParameter {
    fn get_id(&self) -> DialogId { DialogId::SELECT_WEATHER }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}
