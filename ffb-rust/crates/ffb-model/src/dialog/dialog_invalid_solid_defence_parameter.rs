use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogInvalidSolidDefenceParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogInvalidSolidDefenceParameter {
    pub team_id: Option<String>,
    pub amount: i32,
    pub limit: i32,
}

impl DialogInvalidSolidDefenceParameter {
    pub fn get_team_id(&self) -> Option<&str> { self.team_id.as_deref() }
    pub fn get_amount(&self) -> i32 { self.amount }
    pub fn get_limit(&self) -> i32 { self.limit }
}

impl IDialogParameter for DialogInvalidSolidDefenceParameter {
    fn get_id(&self) -> DialogId { DialogId::INVALID_SOLID_DEFENCE }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}

// JSON serialization (BACKLOG §O).
impl DialogInvalidSolidDefenceParameter {
    /// Java `initFrom(IFactorySource, JsonValue)`.
    pub fn init_from(&mut self, _rules: crate::enums::Rules, json: &serde_json::Value) -> Result<(), String> {
        use crate::util::java_json as jj;
        let obj = jj::to_json_object(json)?;
        super::util_dialog_parameter::validate_dialog_id_json(DialogId::INVALID_SOLID_DEFENCE, obj)?;
        self.team_id = jj::get_string(obj, "teamId")?;
        self.amount = jj::get_int(obj, "nrOfPlayers")?;
        self.limit = jj::get_int(obj, "nrOfPlayersAllowed")?;
        Ok(())
    }

    /// Java `toJsonValue()`.
    pub fn to_json_value(&self, _rules: crate::enums::Rules) -> Result<serde_json::Value, String> {
        use crate::util::java_json as jj;
        use serde_json::Value;
        let mut o = serde_json::Map::new();
        o.insert("dialogId".to_string(), Value::String(DialogId::INVALID_SOLID_DEFENCE.get_name().to_string()));
        o.insert("teamId".to_string(), jj::string_or_null(self.team_id.as_deref()));
        o.insert("nrOfPlayers".to_string(), Value::from(self.amount));
        o.insert("nrOfPlayersAllowed".to_string(), Value::from(self.limit));
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
        assert_eq!(round_trip(Rules::Bb2020, json!({"dialogId": "invalidSolidDefence", "nrOfPlayers": 5, "nrOfPlayersAllowed": 4, "teamId": "t1"})).unwrap(), json!({"dialogId": "invalidSolidDefence", "nrOfPlayers": 5, "nrOfPlayersAllowed": 4, "teamId": "t1"}));
    }
}
