use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogUseIgorsParameter.
/// Note: InjuryDescription stored as JSON values (stub not yet translated).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogUseIgorsParameter {
    pub team_id: Option<String>,
    pub injury_descriptions: Vec<serde_json::Value>,
    pub max_igors: i32,
}

impl DialogUseIgorsParameter {
    pub fn get_team_id(&self) -> Option<&str> { self.team_id.as_deref() }
    pub fn get_injury_descriptions(&self) -> &[serde_json::Value] { &self.injury_descriptions }
    pub fn get_max_igors(&self) -> i32 { self.max_igors }
}

impl IDialogParameter for DialogUseIgorsParameter {
    fn get_id(&self) -> DialogId { DialogId::USE_IGORS }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}
