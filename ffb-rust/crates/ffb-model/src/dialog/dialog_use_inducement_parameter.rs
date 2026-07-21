use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogUseInducementParameter.
/// Note: InducementType/Card serialized as String name (stubs not yet translated).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogUseInducementParameter {
    pub team_id: Option<String>,
    /// InducementType[] serialized as names.
    pub inducement_types: Vec<String>,
    /// Card[] serialized as names.
    pub cards: Vec<String>,
    pub player_id: Option<String>,
}

impl DialogUseInducementParameter {
    pub fn get_team_id(&self) -> Option<&str> { self.team_id.as_deref() }
    pub fn get_inducement_types(&self) -> &[String] { &self.inducement_types }
    pub fn get_cards(&self) -> &[String] { &self.cards }
    pub fn get_player_id(&self) -> Option<&str> { self.player_id.as_deref() }
}

impl IDialogParameter for DialogUseInducementParameter {
    fn get_id(&self) -> DialogId { DialogId::USE_INDUCEMENT }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}
