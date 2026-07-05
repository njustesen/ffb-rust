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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dialog_id_is_select_weather() {
        assert_eq!(DialogSelectWeatherParameter::default().get_id(), DialogId::SELECT_WEATHER);
    }
    #[test]
    fn options_map_is_accessible() {
        let mut p = DialogSelectWeatherParameter::default();
        p.options.insert("sunny".into(), 1);
        assert_eq!(p.get_options().get("sunny"), Some(&1));
    }
}
