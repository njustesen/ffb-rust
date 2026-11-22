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

// JSON serialization (BACKLOG §O).
impl DialogTeamSetupParameter {
    /// Java `initFrom(IFactorySource, JsonValue)`.
    pub fn init_from(&mut self, _rules: crate::enums::Rules, _json: &serde_json::Value) -> Result<(), String> {
        Err("DialogTeamSetupParameter.initFrom: not ported yet".to_string())
    }

    /// Java `toJsonValue()`.
    pub fn to_json_value(&self, _rules: crate::enums::Rules) -> Result<serde_json::Value, String> {
        Err("DialogTeamSetupParameter.toJsonValue: not ported yet".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn add_setup_name_filters_empty() {
        let mut p = DialogTeamSetupParameter::default();
        p.add_setup_name("MySetup");
        p.add_setup_name("");
        assert_eq!(p.get_setup_names().len(), 1);
        assert_eq!(p.get_setup_names()[0], "MySetup");
    }
}
