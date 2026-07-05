use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogUseMortuaryAssistantsParameter.
/// Note: InjuryDescription stored as JSON values (stub not yet translated).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogUseMortuaryAssistantsParameter {
    pub team_id: Option<String>,
    pub injury_descriptions: Vec<serde_json::Value>,
    pub max_mortuary_assistants: i32,
}

impl DialogUseMortuaryAssistantsParameter {
    pub fn get_team_id(&self) -> Option<&str> { self.team_id.as_deref() }
    pub fn get_injury_descriptions(&self) -> &[serde_json::Value] { &self.injury_descriptions }
    pub fn get_max_mortuary_assistants(&self) -> i32 { self.max_mortuary_assistants }
}

impl IDialogParameter for DialogUseMortuaryAssistantsParameter {
    fn get_id(&self) -> DialogId { DialogId::USE_MORTUARY_ASSISTANTS }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dialog_id_is_use_mortuary_assistants() {
        assert_eq!(DialogUseMortuaryAssistantsParameter::default().get_id(), DialogId::USE_MORTUARY_ASSISTANTS);
    }
    #[test]
    fn stores_max_mortuary_assistants() {
        let p = DialogUseMortuaryAssistantsParameter { max_mortuary_assistants: 3, ..Default::default() };
        assert_eq!(p.get_max_mortuary_assistants(), 3);
    }
}
