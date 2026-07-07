use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogWizardSpellParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogWizardSpellParameter {
    pub team_id: Option<String>,
}

impl DialogWizardSpellParameter {
    pub fn get_team_id(&self) -> Option<&str> { self.team_id.as_deref() }
}

impl IDialogParameter for DialogWizardSpellParameter {
    fn get_id(&self) -> DialogId { DialogId::WIZARD_SPELL }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dialog_id_is_wizard_spell() {
        assert_eq!(DialogWizardSpellParameter::default().get_id(), DialogId::WIZARD_SPELL);
    }
    #[test]
    fn stores_team_id() {
        let p = DialogWizardSpellParameter { team_id: Some("home".into()) };
        assert_eq!(p.get_team_id(), Some("home"));
    }
    #[test]
    fn default_is_sensible() {
        let p = DialogWizardSpellParameter::default();
        assert!(p.get_team_id().is_none());
    }
    #[test]
    fn transform_preserves_id() {
        let p = DialogWizardSpellParameter { team_id: Some("away".into()) };
        assert_eq!(p.transform().get_id(), DialogId::WIZARD_SPELL);
    }
    #[test]
    fn team_id_none_when_unset() {
        let p = DialogWizardSpellParameter { team_id: None };
        assert!(p.get_team_id().is_none());
    }
}
