use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogWinningsReRollParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogWinningsReRollParameter {
    pub team_id: Option<String>,
    pub old_roll: i32,
}

impl DialogWinningsReRollParameter {
    pub fn get_team_id(&self) -> Option<&str> { self.team_id.as_deref() }
    pub fn get_old_roll(&self) -> i32 { self.old_roll }
}

impl IDialogParameter for DialogWinningsReRollParameter {
    fn get_id(&self) -> DialogId { DialogId::WINNINGS_RE_ROLL }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}

// JSON serialization (BACKLOG §O).
impl DialogWinningsReRollParameter {
    /// Java `initFrom(IFactorySource, JsonValue)`.
    pub fn init_from(&mut self, _rules: crate::enums::Rules, json: &serde_json::Value) -> Result<(), String> {
        use crate::util::java_json as jj;
        let obj = jj::to_json_object(json)?;
        super::util_dialog_parameter::validate_dialog_id_json(DialogId::WINNINGS_RE_ROLL, obj)?;
        self.team_id = jj::get_string(obj, "teamId")?;
        self.old_roll = jj::get_int(obj, "oldRoll")?;
        Ok(())
    }

    /// Java `toJsonValue()`.
    pub fn to_json_value(&self, _rules: crate::enums::Rules) -> Result<serde_json::Value, String> {
        use crate::util::java_json as jj;
        use serde_json::Value;
        let mut o = serde_json::Map::new();
        o.insert("dialogId".to_string(), Value::String(DialogId::WINNINGS_RE_ROLL.get_name().to_string()));
        o.insert("teamId".to_string(), jj::string_or_null(self.team_id.as_deref()));
        o.insert("oldRoll".to_string(), Value::from(self.old_roll));
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
        assert_eq!(round_trip(Rules::Bb2016, json!({"dialogId": "winningsReRoll", "oldRoll": 6, "teamId": "t1"})).unwrap(), json!({"dialogId": "winningsReRoll", "oldRoll": 6, "teamId": "t1"}));
    }

    #[test]
    fn null_team_is_written_as_null() {
        assert_eq!(round_trip(Rules::Bb2016, json!({"dialogId": "winningsReRoll"})).unwrap(), json!({"dialogId": "winningsReRoll", "oldRoll": 0, "teamId": null}));
    }
}
