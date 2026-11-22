use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogSwarmingErrorParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogSwarmingErrorParameter {
    pub allowed: i32,
    pub actual: i32,
}

impl DialogSwarmingErrorParameter {
    pub fn get_allowed(&self) -> i32 { self.allowed }
    pub fn get_actual(&self) -> i32 { self.actual }
}

impl IDialogParameter for DialogSwarmingErrorParameter {
    fn get_id(&self) -> DialogId { DialogId::SWARMING_ERROR }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}

// JSON serialization (BACKLOG §O).
impl DialogSwarmingErrorParameter {
    /// Java `initFrom(IFactorySource, JsonValue)`.
    pub fn init_from(&mut self, _rules: crate::enums::Rules, json: &serde_json::Value) -> Result<(), String> {
        use crate::util::java_json as jj;
        let obj = jj::to_json_object(json)?;
        super::util_dialog_parameter::validate_dialog_id_json(DialogId::SWARMING_ERROR, obj)?;
        self.allowed = jj::get_int(obj, "swarmingPlayerAllowed")?;
        self.actual = jj::get_int(obj, "swarmingPlayerActual")?;
        Ok(())
    }

    /// Java `toJsonValue()`.
    pub fn to_json_value(&self, _rules: crate::enums::Rules) -> Result<serde_json::Value, String> {
        use serde_json::Value;
        let mut o = serde_json::Map::new();
        o.insert("dialogId".to_string(), Value::String(DialogId::SWARMING_ERROR.get_name().to_string()));
        o.insert("swarmingPlayerAllowed".to_string(), Value::from(self.allowed));
        o.insert("swarmingPlayerActual".to_string(), Value::from(self.actual));
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
        assert_eq!(round_trip(Rules::Bb2025, json!({"dialogId": "swarmingError", "swarmingPlayerActual": 2, "swarmingPlayerAllowed": 1})).unwrap(), json!({"dialogId": "swarmingError", "swarmingPlayerActual": 2, "swarmingPlayerAllowed": 1}));
    }
}
