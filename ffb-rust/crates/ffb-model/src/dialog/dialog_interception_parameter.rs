use serde::{Deserialize, Serialize};
use crate::enums::SkillId;
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogInterceptionParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogInterceptionParameter {
    pub thrower_id: Option<String>,
    pub interception_skill: Option<SkillId>,
    pub skill_mnemonic: i32,
}

impl DialogInterceptionParameter {
    pub fn get_thrower_id(&self) -> Option<&str> { self.thrower_id.as_deref() }
    pub fn get_interception_skill(&self) -> Option<SkillId> { self.interception_skill }
    pub fn get_skill_mnemonic(&self) -> i32 { self.skill_mnemonic }
}

impl IDialogParameter for DialogInterceptionParameter {
    fn get_id(&self) -> DialogId { DialogId::INTERCEPTION }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}
