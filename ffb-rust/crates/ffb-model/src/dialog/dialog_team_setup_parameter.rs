use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogTeamSetupParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogTeamSetupParameter {
    pub load_dialog: bool,
    pub setup_names: Vec<String>,
}

impl DialogTeamSetupParameter {
    pub fn is_load_dialog(&self) -> bool { self.load_dialog }
    pub fn get_setup_names(&self) -> &[String] { &self.setup_names }
    pub fn add_setup_name(&mut self, name: impl Into<String>) {
        let s = name.into();
        if !s.is_empty() { self.setup_names.push(s); }
    }
}

impl IDialogParameter for DialogTeamSetupParameter {
    fn get_id(&self) -> DialogId { DialogId::TEAM_SETUP }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dialog_id_is_team_setup() {
        assert_eq!(DialogTeamSetupParameter::default().get_id(), DialogId::TEAM_SETUP);
    }
    #[test]
    fn add_setup_name_filters_empty() {
        let mut p = DialogTeamSetupParameter::default();
        p.add_setup_name("MySetup");
        p.add_setup_name("");
        assert_eq!(p.get_setup_names().len(), 1);
        assert_eq!(p.get_setup_names()[0], "MySetup");
    }
    #[test]
    fn default_is_sensible() {
        let p = DialogTeamSetupParameter::default();
        assert!(!p.is_load_dialog());
        assert!(p.get_setup_names().is_empty());
    }
    #[test]
    fn load_dialog_flag_readable() {
        let p = DialogTeamSetupParameter { load_dialog: true, setup_names: vec!["A".into(), "B".into()] };
        assert!(p.is_load_dialog());
        assert_eq!(p.get_setup_names().len(), 2);
    }
    #[test]
    fn multiple_names_added() {
        let mut p = DialogTeamSetupParameter::default();
        p.add_setup_name("Setup1");
        p.add_setup_name("Setup2");
        p.add_setup_name("");
        assert_eq!(p.get_setup_names().len(), 2);
    }
}
