use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogUseMortuaryAssistantParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogUseMortuaryAssistantParameter {
    pub player_id: Option<String>,
}

impl DialogUseMortuaryAssistantParameter {
    pub fn get_player_id(&self) -> Option<&str> { self.player_id.as_deref() }
}

impl IDialogParameter for DialogUseMortuaryAssistantParameter {
    fn get_id(&self) -> DialogId { DialogId::USE_MORTUARY_ASSISTANT }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dialog_id_is_use_mortuary_assistant() {
        assert_eq!(DialogUseMortuaryAssistantParameter::default().get_id(), DialogId::USE_MORTUARY_ASSISTANT);
    }
    #[test]
    fn stores_player_id() {
        let p = DialogUseMortuaryAssistantParameter { player_id: Some("p1".into()) };
        assert_eq!(p.get_player_id(), Some("p1"));
    }
}
