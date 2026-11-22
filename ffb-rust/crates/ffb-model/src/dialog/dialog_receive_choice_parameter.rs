use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogReceiveChoiceParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogReceiveChoiceParameter {
    pub choosing_team_id: Option<String>,
}

impl DialogReceiveChoiceParameter {
    pub fn get_choosing_team_id(&self) -> Option<&str> { self.choosing_team_id.as_deref() }
}

impl IDialogParameter for DialogReceiveChoiceParameter {
    fn get_id(&self) -> DialogId { DialogId::RECEIVE_CHOICE }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}

// JSON serialization (BACKLOG §O).
impl DialogReceiveChoiceParameter {
    /// Java `initFrom(IFactorySource, JsonValue)`.
    pub fn init_from(&mut self, _rules: crate::enums::Rules, json: &serde_json::Value) -> Result<(), String> {
        use crate::util::java_json as jj;
        let obj = jj::to_json_object(json)?;
        super::util_dialog_parameter::validate_dialog_id_json(DialogId::RECEIVE_CHOICE, obj)?;
        self.choosing_team_id = jj::get_string(obj, "choosingTeamId")?;
        Ok(())
    }

    /// Java `toJsonValue()`.
    pub fn to_json_value(&self, _rules: crate::enums::Rules) -> Result<serde_json::Value, String> {
        use crate::util::java_json as jj;
        use serde_json::Value;
        let mut o = serde_json::Map::new();
        o.insert("dialogId".to_string(), Value::String(DialogId::RECEIVE_CHOICE.get_name().to_string()));
        o.insert("choosingTeamId".to_string(), jj::string_or_null(self.choosing_team_id.as_deref()));
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
        assert_eq!(round_trip(Rules::Bb2025, json!({"choosingTeamId": "t1", "dialogId": "receiveChoice"})).unwrap(), json!({"choosingTeamId": "t1", "dialogId": "receiveChoice"}));
    }

    #[test]
    fn non_string_team_throws() {
        assert!(round_trip(Rules::Bb2025, json!({"choosingTeamId": 5, "dialogId": "receiveChoice"})).is_err());
    }
}
