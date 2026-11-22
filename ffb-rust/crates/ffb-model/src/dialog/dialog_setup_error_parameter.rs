use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogSetupErrorParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogSetupErrorParameter {
    pub team_id: Option<String>,
    pub setup_errors: Vec<String>,
}

impl DialogSetupErrorParameter {
    pub fn get_team_id(&self) -> Option<&str> { self.team_id.as_deref() }
    pub fn get_setup_errors(&self) -> &[String] { &self.setup_errors }
    pub fn add_setup_error(&mut self, error: impl Into<String>) {
        let s = error.into();
        if !s.is_empty() { self.setup_errors.push(s); }
    }
}

impl IDialogParameter for DialogSetupErrorParameter {
    fn get_id(&self) -> DialogId { DialogId::SETUP_ERROR }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}

// JSON serialization (BACKLOG §O).
impl DialogSetupErrorParameter {
    /// Java `addXxx(String[])`: `ArrayTool.isProvided` (a null or empty array adds nothing), each
    /// element through the `StringTool.isProvided` filter (null and "" are skipped).
    fn add_provided(target: &mut Vec<String>, values: Option<Vec<Option<String>>>) {
        for value in values.into_iter().flatten().flatten() {
            if crate::util::java_json::is_provided(Some(&value)) {
                target.push(value);
            }
        }
    }

    /// Java `initFrom(IFactorySource, JsonValue)`.
    pub fn init_from(&mut self, _rules: crate::enums::Rules, json: &serde_json::Value) -> Result<(), String> {
        use crate::util::java_json as jj;
        let obj = jj::to_json_object(json)?;
        super::util_dialog_parameter::validate_dialog_id_json(DialogId::SETUP_ERROR, obj)?;
        self.team_id = jj::get_string(obj, "teamId")?;
        Self::add_provided(&mut self.setup_errors, jj::get_string_array(obj, "setupErrors")?);
        Ok(())
    }

    /// Java `toJsonValue()`.
    pub fn to_json_value(&self, _rules: crate::enums::Rules) -> Result<serde_json::Value, String> {
        use crate::util::java_json as jj;
        use serde_json::Value;
        let mut o = serde_json::Map::new();
        o.insert("dialogId".to_string(), Value::String(DialogId::SETUP_ERROR.get_name().to_string()));
        o.insert("teamId".to_string(), jj::string_or_null(self.team_id.as_deref()));
        o.insert("setupErrors".to_string(), jj::string_array(&self.setup_errors));
        Ok(Value::Object(o))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn add_setup_error_filters_empty() {
        let mut p = DialogSetupErrorParameter::default();
        p.add_setup_error("err1");
        p.add_setup_error("");
        assert_eq!(p.get_setup_errors().len(), 1);
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
        assert_eq!(round_trip(Rules::Bb2020, json!({"dialogId": "setupError", "setupErrors": ["a", "b"], "teamId": "t1"})).unwrap(), json!({"dialogId": "setupError", "setupErrors": ["a", "b"], "teamId": "t1"}));
    }

    #[test]
    fn missing_errors_is_empty() {
        assert_eq!(round_trip(Rules::Bb2020, json!({"dialogId": "setupError", "setupErrors": null})).unwrap(), json!({"dialogId": "setupError", "setupErrors": [], "teamId": null}));
    }
}
