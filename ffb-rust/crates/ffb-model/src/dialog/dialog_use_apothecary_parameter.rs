use serde::{Deserialize, Serialize};
use crate::enums::{PlayerState, SeriousInjuryKind, ApothecaryType};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogUseApothecaryParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogUseApothecaryParameter {
    pub player_id: Option<String>,
    pub player_state: Option<PlayerState>,
    pub serious_injury: Option<SeriousInjuryKind>,
    pub apothecary_types: Vec<ApothecaryType>,
}

impl DialogUseApothecaryParameter {
    pub fn get_player_id(&self) -> Option<&str> { self.player_id.as_deref() }
    pub fn get_player_state(&self) -> Option<PlayerState> { self.player_state }
    pub fn get_serious_injury(&self) -> Option<SeriousInjuryKind> { self.serious_injury }
    pub fn get_apothecary_types(&self) -> &[ApothecaryType] { &self.apothecary_types }
}

impl IDialogParameter for DialogUseApothecaryParameter {
    fn get_id(&self) -> DialogId { DialogId::USE_APOTHECARY }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}
