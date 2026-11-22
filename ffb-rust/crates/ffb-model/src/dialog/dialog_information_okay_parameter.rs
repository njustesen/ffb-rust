use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogInformationOkayParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogInformationOkayParameter {
    pub title: Option<String>,
    /// Java `String[]`: `None` = a null array, an inner `None` = a null element.
    pub messages: Option<Vec<Option<String>>>,
    pub confirm: bool,
}

impl DialogInformationOkayParameter {
    pub fn get_title(&self) -> Option<&str> { self.title.as_deref() }
    pub fn get_messages(&self) -> Option<&[Option<String>]> { self.messages.as_deref() }
    pub fn is_confirm(&self) -> bool { self.confirm }
}

impl IDialogParameter for DialogInformationOkayParameter {
    fn get_id(&self) -> DialogId { DialogId::INFORMATION_OKAY }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}

// JSON serialization (BACKLOG §O).
impl DialogInformationOkayParameter {
    /// Java `initFrom(IFactorySource, JsonValue)`.
    pub fn init_from(&mut self, _rules: crate::enums::Rules, json: &serde_json::Value) -> Result<(), String> {
        use crate::util::java_json as jj;
        let obj = jj::to_json_object(json)?;
        super::util_dialog_parameter::validate_dialog_id_json(DialogId::INFORMATION_OKAY, obj)?;
        self.title = jj::get_string(obj, "text")?;
        self.messages = jj::get_string_array(obj, "messageArray")?;
        // a primitive `boolean` from a `Boolean`: absent -> NullPointerException.
        self.confirm = jj::get_boolean(obj, "confirm")?;
        Ok(())
    }

    /// Java `toJsonValue()`.
    pub fn to_json_value(&self, _rules: crate::enums::Rules) -> Result<serde_json::Value, String> {
        use crate::util::java_json as jj;
        use serde_json::Value;
        let mut o = serde_json::Map::new();
        o.insert("dialogId".to_string(), Value::String(DialogId::INFORMATION_OKAY.get_name().to_string()));
        o.insert("messageArray".to_string(), jj::string_array_or_null(self.messages.as_deref()));
        o.insert("text".to_string(), jj::string_or_null(self.title.as_deref()));
        o.insert("confirm".to_string(), Value::Bool(self.confirm));
        Ok(Value::Object(o))
    }
}

#[cfg(test)]
mod json_tests {
    use crate::dialog::dialog_parameter_factory::DialogParameterFactory;
    use crate::enums::Rules;
    use serde_json::{json, Value};

    fn round_trip(rules: Rules, input: Value) -> Result<Value, String> {
        let p = DialogParameterFactory::new().for_json_value(rules, &input)?.expect("parameter");
        p.to_json_value(rules)
    }

    #[test]
    fn json_round_trip_case() {
        assert_eq!(round_trip(Rules::Bb2025, json!({"confirm": true, "dialogId": "informationOkay", "messageArray": ["The weather did not change."], "text": "Weather Mage"})).unwrap(), json!({"confirm": true, "dialogId": "informationOkay", "messageArray": ["The weather did not change."], "text": "Weather Mage"}));
    }

    #[test]
    fn null_messages_and_title() {
        assert_eq!(round_trip(Rules::Bb2025, json!({"confirm": false, "dialogId": "informationOkay", "messageArray": [null]})).unwrap(), json!({"confirm": false, "dialogId": "informationOkay", "messageArray": [null], "text": null}));
    }

    #[test]
    fn absent_messages_are_null() {
        assert_eq!(round_trip(Rules::Bb2025, json!({"confirm": false, "dialogId": "informationOkay"})).unwrap(), json!({"confirm": false, "dialogId": "informationOkay", "messageArray": null, "text": null}));
    }

    #[test]
    fn missing_confirm_throws() {
        assert!(round_trip(Rules::Bb2025, json!({"dialogId": "informationOkay"})).is_err());
    }
}
