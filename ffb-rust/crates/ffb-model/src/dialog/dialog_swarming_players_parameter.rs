use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogSwarmingPlayersParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogSwarmingPlayersParameter {
    pub amount: i32,
    pub restrict_placement: bool,
}

impl DialogSwarmingPlayersParameter {
    pub fn get_amount(&self) -> i32 { self.amount }
    pub fn is_restrict_placement(&self) -> bool { self.restrict_placement }
}

impl IDialogParameter for DialogSwarmingPlayersParameter {
    fn get_id(&self) -> DialogId { DialogId::SWARMING }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}

// JSON serialization (BACKLOG §O).
impl DialogSwarmingPlayersParameter {
    /// Java `initFrom(IFactorySource, JsonValue)`.
    pub fn init_from(&mut self, _rules: crate::enums::Rules, json: &serde_json::Value) -> Result<(), String> {
        use crate::util::java_json as jj;
        let obj = jj::to_json_object(json)?;
        super::util_dialog_parameter::validate_dialog_id_json(DialogId::SWARMING, obj)?;
        self.amount = jj::get_int(obj, "swarmingPlayerAmount")?;
        if jj::is_defined_in(obj, "restrictPlacement") {
            self.restrict_placement = jj::get_boolean(obj, "restrictPlacement")?;
        } else {
            self.restrict_placement = true;
        }
        Ok(())
    }

    /// Java `toJsonValue()`.
    pub fn to_json_value(&self, _rules: crate::enums::Rules) -> Result<serde_json::Value, String> {
        use serde_json::Value;
        let mut o = serde_json::Map::new();
        o.insert("dialogId".to_string(), Value::String(DialogId::SWARMING.get_name().to_string()));
        o.insert("swarmingPlayerAmount".to_string(), Value::from(self.amount));
        o.insert("restrictPlacement".to_string(), Value::Bool(self.restrict_placement));
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
        assert_eq!(round_trip(Rules::Bb2025, json!({"dialogId": "swarming", "restrictPlacement": false, "swarmingPlayerAmount": 3})).unwrap(), json!({"dialogId": "swarming", "restrictPlacement": false, "swarmingPlayerAmount": 3}));
    }

    #[test]
    fn missing_restrict_placement_defaults_true() {
        assert_eq!(round_trip(Rules::Bb2016, json!({"dialogId": "swarming", "swarmingPlayerAmount": 1})).unwrap(), json!({"dialogId": "swarming", "restrictPlacement": true, "swarmingPlayerAmount": 1}));
    }
}
