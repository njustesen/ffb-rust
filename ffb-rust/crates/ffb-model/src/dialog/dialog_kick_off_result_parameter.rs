use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogKickOffResultParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogKickOffResultParameter {
    pub team_id: Option<String>,
}

impl DialogKickOffResultParameter {
    pub fn get_team_id(&self) -> Option<&str> { self.team_id.as_deref() }
}

impl IDialogParameter for DialogKickOffResultParameter {
    fn get_id(&self) -> DialogId { DialogId::KICK_OFF_RESULT }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn serde_round_trip() {
        let p = DialogKickOffResultParameter { team_id: Some("home".into()) };
        let json = serde_json::to_string(&p).unwrap();
        let back: DialogKickOffResultParameter = serde_json::from_str(&json).unwrap();
        assert_eq!(back.get_team_id(), Some("home"));
    }

}
