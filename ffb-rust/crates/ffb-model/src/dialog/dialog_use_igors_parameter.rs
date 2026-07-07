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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dialog_id_is_use_igors() {
        assert_eq!(DialogUseIgorsParameter::default().get_id(), DialogId::USE_IGORS);
    }
    #[test]
    fn stores_team_id_and_max_igors() {
        let p = DialogUseIgorsParameter { team_id: Some("home".into()), max_igors: 2, ..Default::default() };
        assert_eq!(p.get_team_id(), Some("home"));
        assert_eq!(p.get_max_igors(), 2);
    }
    #[test]
    fn default_is_sensible() {
        let p = DialogUseIgorsParameter::default();
        assert!(p.get_team_id().is_none());
        assert_eq!(p.get_max_igors(), 0);
        assert!(p.get_injury_descriptions().is_empty());
    }
    #[test]
    fn stores_injury_descriptions() {
        let p = DialogUseIgorsParameter {
            injury_descriptions: vec![serde_json::json!({"player": "p3"})],
            ..Default::default()
        };
        assert_eq!(p.get_injury_descriptions().len(), 1);
    }
    #[test]
    fn team_id_none_when_unset() {
        let p = DialogUseIgorsParameter { team_id: None, ..Default::default() };
        assert!(p.get_team_id().is_none());
    }
}
