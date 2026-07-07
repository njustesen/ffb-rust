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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dialog_id_is_interception() {
        assert_eq!(DialogInterceptionParameter::default().get_id(), DialogId::INTERCEPTION);
    }

    #[test]
    fn stores_thrower_id_and_skill() {
        let p = DialogInterceptionParameter {
            thrower_id: Some("t1".into()),
            interception_skill: Some(SkillId::Catch),
            skill_mnemonic: 42,
        };
        assert_eq!(p.get_thrower_id(), Some("t1"));
        assert_eq!(p.get_interception_skill(), Some(SkillId::Catch));
        assert_eq!(p.get_skill_mnemonic(), 42);
    }

    #[test]
    fn default_is_sensible() {
        let p = DialogInterceptionParameter::default();
        assert!(p.get_thrower_id().is_none());
        assert!(p.get_interception_skill().is_none());
        assert_eq!(p.get_skill_mnemonic(), 0);
    }

    #[test]
    fn accessor_methods_with_non_default_values() {
        let p = DialogInterceptionParameter {
            thrower_id: Some("player42".into()),
            interception_skill: Some(SkillId::DivingCatch),
            skill_mnemonic: 7,
        };
        assert_eq!(p.get_thrower_id(), Some("player42"));
        assert_eq!(p.get_interception_skill(), Some(SkillId::DivingCatch));
        assert_eq!(p.get_skill_mnemonic(), 7);
    }

    #[test]
    fn none_interception_skill_is_edge_case() {
        let p = DialogInterceptionParameter {
            thrower_id: Some("p1".into()),
            interception_skill: None,
            skill_mnemonic: 0,
        };
        assert!(p.get_interception_skill().is_none());
        assert_eq!(p.get_skill_mnemonic(), 0);
    }
}
