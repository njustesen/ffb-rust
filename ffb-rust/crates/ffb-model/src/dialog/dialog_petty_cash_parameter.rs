use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogPettyCashParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogPettyCashParameter {
    pub team_id: Option<String>,
    pub treasury: i32,
    pub team_value: i32,
    pub opponent_team_value: i32,
}

impl DialogPettyCashParameter {
    pub fn get_team_id(&self) -> Option<&str> { self.team_id.as_deref() }
    pub fn get_treasury(&self) -> i32 { self.treasury }
    pub fn get_team_value(&self) -> i32 { self.team_value }
    pub fn get_opponent_team_value(&self) -> i32 { self.opponent_team_value }
}

impl IDialogParameter for DialogPettyCashParameter {
    fn get_id(&self) -> DialogId { DialogId::PETTY_CASH }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}

// JSON serialization (BACKLOG §O).
impl DialogPettyCashParameter {
    /// Java `initFrom(IFactorySource, JsonValue)`.
    pub fn init_from(&mut self, _rules: crate::enums::Rules, json: &serde_json::Value) -> Result<(), String> {
        use crate::util::java_json as jj;
        let obj = jj::to_json_object(json)?;
        super::util_dialog_parameter::validate_dialog_id_json(DialogId::PETTY_CASH, obj)?;
        self.team_id = jj::get_string(obj, "teamId")?;
        self.team_value = jj::get_int(obj, "teamValue")?;
        self.treasury = jj::get_int(obj, "treasury")?;
        self.opponent_team_value = jj::get_int(obj, "opponentTeamValue")?;
        Ok(())
    }

    /// Java `toJsonValue()`.
    pub fn to_json_value(&self, _rules: crate::enums::Rules) -> Result<serde_json::Value, String> {
        use crate::util::java_json as jj;
        use serde_json::Value;
        let mut o = serde_json::Map::new();
        o.insert("dialogId".to_string(), Value::String(DialogId::PETTY_CASH.get_name().to_string()));
        o.insert("teamId".to_string(), jj::string_or_null(self.team_id.as_deref()));
        o.insert("teamValue".to_string(), Value::from(self.team_value));
        o.insert("treasury".to_string(), Value::from(self.treasury));
        o.insert("opponentTeamValue".to_string(), Value::from(self.opponent_team_value));
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
        assert_eq!(round_trip(Rules::Bb2016, json!({"dialogId": "pettyCash", "opponentTeamValue": 1200000, "teamId": "t1", "teamValue": 1280000, "treasury": 120000})).unwrap(), json!({"dialogId": "pettyCash", "opponentTeamValue": 1200000, "teamId": "t1", "teamValue": 1280000, "treasury": 120000}));
    }
}
