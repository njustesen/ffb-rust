use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogBuyInducementsParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogBuyInducementsParameter {
    pub team_id: Option<String>,
    pub available_gold: i32,
}

impl DialogBuyInducementsParameter {
    pub fn get_team_id(&self) -> Option<&str> { self.team_id.as_deref() }
    pub fn get_available_gold(&self) -> i32 { self.available_gold }
}

impl IDialogParameter for DialogBuyInducementsParameter {
    fn get_id(&self) -> DialogId { DialogId::BUY_INDUCEMENTS }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}

// JSON serialization (BACKLOG §O).
impl DialogBuyInducementsParameter {
    /// Java `initFrom(IFactorySource, JsonValue)`.
    pub fn init_from(&mut self, _rules: crate::enums::Rules, json: &serde_json::Value) -> Result<(), String> {
        use crate::util::java_json as jj;
        let obj = jj::to_json_object(json)?;
        super::util_dialog_parameter::validate_dialog_id_json(DialogId::BUY_INDUCEMENTS, obj)?;
        self.team_id = jj::get_string(obj, "teamId")?;
        self.available_gold = jj::get_int(obj, "availableGold")?;
        Ok(())
    }

    /// Java `toJsonValue()`.
    pub fn to_json_value(&self, _rules: crate::enums::Rules) -> Result<serde_json::Value, String> {
        use crate::util::java_json as jj;
        use serde_json::Value;
        let mut o = serde_json::Map::new();
        o.insert("dialogId".to_string(), Value::String(DialogId::BUY_INDUCEMENTS.get_name().to_string()));
        o.insert("teamId".to_string(), jj::string_or_null(self.team_id.as_deref()));
        o.insert("availableGold".to_string(), Value::from(self.available_gold));
        Ok(Value::Object(o))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_round_trip() {
        let p = DialogBuyInducementsParameter { team_id: Some("teamY".into()), available_gold: 200_000 };
        let json = serde_json::to_string(&p).unwrap();
        let back: DialogBuyInducementsParameter = serde_json::from_str(&json).unwrap();
        assert_eq!(back.get_team_id(), Some("teamY"));
        assert_eq!(back.get_available_gold(), 200_000);
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
        assert_eq!(round_trip(Rules::Bb2016, json!({"dialogId": "buyInducements", "teamId": "t1", "availableGold": 80000})).unwrap(), json!({"dialogId": "buyInducements", "teamId": "t1", "availableGold": 80000}));
    }

    #[test]
    fn non_int_gold_throws() {
        assert!(round_trip(Rules::Bb2016, json!({"dialogId": "buyInducements", "availableGold": "x"})).is_err());
    }
}
