use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogFollowupChoiceParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogFollowupChoiceParameter;

impl IDialogParameter for DialogFollowupChoiceParameter {
    fn get_id(&self) -> DialogId { DialogId::FOLLOWUP_CHOICE }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}

// JSON serialization (BACKLOG §O): Java `initFrom` validates the dialog id only; `toJsonValue` is
// `DialogWithoutParameter.toJsonValue` (the dialog id only).
impl DialogFollowupChoiceParameter {
    /// Java `initFrom(IFactorySource, JsonValue)`.
    pub fn init_from(&mut self, _rules: crate::enums::Rules, json: &serde_json::Value) -> Result<(), String> {
        let obj = crate::util::java_json::to_json_object(json)?;
        super::util_dialog_parameter::validate_dialog_id_json(DialogId::FOLLOWUP_CHOICE, obj)
    }

    /// Java `toJsonValue()`.
    pub fn to_json_value(&self, _rules: crate::enums::Rules) -> Result<serde_json::Value, String> {
        Ok(super::dialog_without_parameter::DialogWithoutParameter::to_json_value(DialogId::FOLLOWUP_CHOICE))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn serde_round_trip() {
        let p = DialogFollowupChoiceParameter;
        let json = serde_json::to_string(&p).unwrap();
        let back: DialogFollowupChoiceParameter = serde_json::from_str(&json).unwrap();
        assert_eq!(back.get_id(), DialogId::FOLLOWUP_CHOICE);
    }

}
