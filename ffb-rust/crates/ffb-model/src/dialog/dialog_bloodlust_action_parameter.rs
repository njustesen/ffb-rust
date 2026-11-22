use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogBloodlustActionParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogBloodlustActionParameter {
    pub change_to_move: bool,
}

impl DialogBloodlustActionParameter {
    pub fn is_change_to_move(&self) -> bool { self.change_to_move }
}

impl IDialogParameter for DialogBloodlustActionParameter {
    fn get_id(&self) -> DialogId { DialogId::BLOODLUST_ACTION }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}

// JSON serialization (BACKLOG §O).
impl DialogBloodlustActionParameter {
    /// Java `initFrom(IFactorySource, JsonValue)`.
    pub fn init_from(&mut self, _rules: crate::enums::Rules, json: &serde_json::Value) -> Result<(), String> {
        use crate::util::java_json as jj;
        let obj = jj::to_json_object(json)?;
        super::util_dialog_parameter::validate_dialog_id_json(DialogId::BLOODLUST_ACTION, obj)?;
        // JsonBooleanOption into a primitive boolean: absent -> NPE.
        self.change_to_move = jj::get_boolean(obj, "changeToMove")?;
        Ok(())
    }

    /// Java `toJsonValue()` = `DialogWithoutParameter.toJsonValue()` (the dialog id) + changeToMove.
    pub fn to_json_value(&self, _rules: crate::enums::Rules) -> Result<serde_json::Value, String> {
        use serde_json::Value;
        let mut o = serde_json::Map::new();
        o.insert("dialogId".into(), Value::from(DialogId::BLOODLUST_ACTION.get_name()));
        o.insert("changeToMove".into(), Value::Bool(self.change_to_move));
        Ok(Value::Object(o))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_round_trip() {
        let p = DialogBloodlustActionParameter { change_to_move: true };
        let json = serde_json::to_string(&p).unwrap();
        let back: DialogBloodlustActionParameter = serde_json::from_str(&json).unwrap();
        assert!(back.is_change_to_move());
    }

}
