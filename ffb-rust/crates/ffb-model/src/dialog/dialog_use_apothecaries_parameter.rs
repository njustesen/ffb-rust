use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogUseApothecariesParameter.
/// Note: InjuryDescription stored as JSON values (stub not yet translated).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogUseApothecariesParameter {
    pub team_id: Option<String>,
    pub injury_descriptions: Vec<serde_json::Value>,
}

impl DialogUseApothecariesParameter {
    pub fn get_team_id(&self) -> Option<&str> { self.team_id.as_deref() }
    pub fn get_injury_descriptions(&self) -> &[serde_json::Value] { &self.injury_descriptions }
}

impl IDialogParameter for DialogUseApothecariesParameter {
    fn get_id(&self) -> DialogId { DialogId::USE_APOTHECARIES }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}
