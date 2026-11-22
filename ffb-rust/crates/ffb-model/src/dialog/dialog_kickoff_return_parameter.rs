use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogKickoffReturnParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogKickoffReturnParameter;

impl IDialogParameter for DialogKickoffReturnParameter {
    fn get_id(&self) -> DialogId { DialogId::KICKOFF_RETURN }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}

// JSON serialization (BACKLOG §O): Java `initFrom` validates the dialog id only; `toJsonValue` is
// `DialogWithoutParameter.toJsonValue` (the dialog id only).
impl DialogKickoffReturnParameter {
    /// Java `initFrom(IFactorySource, JsonValue)`.
    pub fn init_from(&mut self, _rules: crate::enums::Rules, json: &serde_json::Value) -> Result<(), String> {
        let obj = crate::util::java_json::to_json_object(json)?;
        super::util_dialog_parameter::validate_dialog_id_json(DialogId::KICKOFF_RETURN, obj)
    }

    /// Java `toJsonValue()`.
    pub fn to_json_value(&self, _rules: crate::enums::Rules) -> Result<serde_json::Value, String> {
        Ok(super::dialog_without_parameter::DialogWithoutParameter::to_json_value(DialogId::KICKOFF_RETURN))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn serde_round_trip() {
        let p = DialogKickoffReturnParameter;
        let json = serde_json::to_string(&p).unwrap();
        let back: DialogKickoffReturnParameter = serde_json::from_str(&json).unwrap();
        assert_eq!(back.get_id(), DialogId::KICKOFF_RETURN);
    }
}
