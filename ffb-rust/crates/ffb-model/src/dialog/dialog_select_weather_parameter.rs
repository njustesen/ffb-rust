use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogSelectWeatherParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogSelectWeatherParameter {
    pub options: HashMap<String, i32>,
}

impl DialogSelectWeatherParameter {
    pub fn get_options(&self) -> &HashMap<String, i32> { &self.options }
}

impl IDialogParameter for DialogSelectWeatherParameter {
    fn get_id(&self) -> DialogId { DialogId::SELECT_WEATHER }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}

// JSON serialization (BACKLOG §O).
impl DialogSelectWeatherParameter {
    /// Java `initFrom(IFactorySource, JsonValue)`.
    pub fn init_from(&mut self, _rules: crate::enums::Rules, json: &serde_json::Value) -> Result<(), String> {
        use crate::util::java_json as jj;
        let obj = jj::to_json_object(json)?;
        super::util_dialog_parameter::validate_dialog_id_json(DialogId::SELECT_WEATHER, obj)?;
        // JsonIntegerMapOption: a new map, filled only from a JSON object (each value `asInt`).
        self.options = jj::get_integer_map(obj, "weatherOptions")?.into_iter().collect();
        Ok(())
    }

    /// Java `toJsonValue()`.
    pub fn to_json_value(&self, _rules: crate::enums::Rules) -> Result<serde_json::Value, String> {
        use crate::util::java_json as jj;
        use serde_json::Value;
        let mut o = serde_json::Map::new();
        o.insert("dialogId".to_string(), Value::String(DialogId::SELECT_WEATHER.get_name().to_string()));
        let options: Vec<(String, i32)> = self.options.iter().map(|(k, v)| (k.clone(), *v)).collect();
        o.insert("weatherOptions".to_string(), jj::integer_map(&options));
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
        assert_eq!(round_trip(Rules::Bb2025, json!({"dialogId": "selectWeather", "weatherOptions": {"NICE": 1, "SWELTERING_HEAT": -1, "VERY_SUNNY": 0}})).unwrap(), json!({"dialogId": "selectWeather", "weatherOptions": {"NICE": 1, "SWELTERING_HEAT": -1, "VERY_SUNNY": 0}}));
    }

    #[test]
    fn non_object_is_empty() {
        assert_eq!(round_trip(Rules::Bb2025, json!({"dialogId": "selectWeather", "weatherOptions": [1]})).unwrap(), json!({"dialogId": "selectWeather", "weatherOptions": {}}));
    }

    #[test]
    fn non_int_value_throws() {
        assert!(round_trip(Rules::Bb2025, json!({"dialogId": "selectWeather", "weatherOptions": {"NICE": "x"}})).is_err());
    }
}
