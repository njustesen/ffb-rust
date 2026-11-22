use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogBuyPrayersAndInducementsParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogBuyPrayersAndInducementsParameter {
    pub team_id: Option<String>,
    pub available_gold: i32,
    pub petty_cash: i32,
    pub treasury: i32,
    pub uses_treasury: bool,
}

impl DialogBuyPrayersAndInducementsParameter {
    pub fn get_team_id(&self) -> Option<&str> { self.team_id.as_deref() }
    pub fn get_available_gold(&self) -> i32 { self.available_gold }
    pub fn get_petty_cash(&self) -> i32 { self.petty_cash }
    pub fn get_treasury(&self) -> i32 { self.treasury }
    pub fn is_uses_treasury(&self) -> bool { self.uses_treasury }
}

impl IDialogParameter for DialogBuyPrayersAndInducementsParameter {
    fn get_id(&self) -> DialogId { DialogId::BUY_PRAYERS_AND_INDUCEMENTS }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}

// JSON serialization (BACKLOG §O).
impl DialogBuyPrayersAndInducementsParameter {
    /// Java `initFrom(IFactorySource, JsonValue)`.
    pub fn init_from(&mut self, _rules: crate::enums::Rules, json: &serde_json::Value) -> Result<(), String> {
        use crate::util::java_json as jj;
        let obj = jj::to_json_object(json)?;
        super::util_dialog_parameter::validate_dialog_id_json(DialogId::BUY_PRAYERS_AND_INDUCEMENTS, obj)?;
        self.team_id = jj::get_string(obj, "teamId")?;
        self.available_gold = jj::get_int(obj, "availableGold")?;
        if jj::is_defined_in(obj, "usesTreasury") {
            self.uses_treasury = jj::get_boolean(obj, "usesTreasury")?;
        }
        self.petty_cash = jj::get_int(obj, "pettyCash")?;
        self.treasury = jj::get_int(obj, "treasury")?;
        Ok(())
    }

    /// Java `toJsonValue()`.
    pub fn to_json_value(&self, _rules: crate::enums::Rules) -> Result<serde_json::Value, String> {
        use crate::util::java_json as jj;
        use serde_json::Value;
        let mut o = serde_json::Map::new();
        o.insert("dialogId".to_string(), Value::String(DialogId::BUY_PRAYERS_AND_INDUCEMENTS.get_name().to_string()));
        o.insert("teamId".to_string(), jj::string_or_null(self.team_id.as_deref()));
        o.insert("availableGold".to_string(), Value::from(self.available_gold));
        o.insert("usesTreasury".to_string(), Value::Bool(self.uses_treasury));
        o.insert("pettyCash".to_string(), Value::from(self.petty_cash));
        o.insert("treasury".to_string(), Value::from(self.treasury));
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
        assert_eq!(round_trip(Rules::Bb2025, json!({"dialogId": "buyPrayersAndInducements", "teamId": "t1", "availableGold": 85000, "pettyCash": 35000, "treasury": 50000, "usesTreasury": false})).unwrap(), json!({"dialogId": "buyPrayersAndInducements", "teamId": "t1", "availableGold": 85000, "pettyCash": 35000, "treasury": 50000, "usesTreasury": false}));
    }

    #[test]
    fn missing_uses_treasury_defaults_false() {
        assert_eq!(round_trip(Rules::Bb2025, json!({"dialogId": "buyPrayersAndInducements", "teamId": "t1"})).unwrap(), json!({"dialogId": "buyPrayersAndInducements", "teamId": "t1", "availableGold": 0, "pettyCash": 0, "treasury": 0, "usesTreasury": false}));
    }

    #[test]
    fn null_uses_treasury_throws() {
        assert!(round_trip(Rules::Bb2025, json!({"dialogId": "buyPrayersAndInducements", "usesTreasury": null})).is_err());
    }
}
