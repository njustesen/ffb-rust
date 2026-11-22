use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogBriberyAndCorruptionParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogBriberyAndCorruptionParameter {
    pub team_id: Option<String>,
}

impl DialogBriberyAndCorruptionParameter {
    pub fn get_team_id(&self) -> Option<&str> { self.team_id.as_deref() }
}

impl IDialogParameter for DialogBriberyAndCorruptionParameter {
    fn get_id(&self) -> DialogId { DialogId::BRIBERY_AND_CORRUPTION_RE_ROLL }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}

// JSON serialization (BACKLOG §O).
impl DialogBriberyAndCorruptionParameter {
    /// Java `initFrom(IFactorySource, JsonValue)`.
    pub fn init_from(&mut self, _rules: crate::enums::Rules, json: &serde_json::Value) -> Result<(), String> {
        use crate::util::java_json as jj;
        let obj = jj::to_json_object(json)?;
        super::util_dialog_parameter::validate_dialog_id_json(DialogId::BRIBERY_AND_CORRUPTION_RE_ROLL, obj)?;
        self.team_id = jj::get_string(obj, "teamId")?;
        Ok(())
    }

    /// Java `toJsonValue()`.
    pub fn to_json_value(&self, _rules: crate::enums::Rules) -> Result<serde_json::Value, String> {
        use crate::util::java_json as jj;
        use serde_json::Value;
        let mut o = serde_json::Map::new();
        o.insert("dialogId".to_string(), Value::String(DialogId::BRIBERY_AND_CORRUPTION_RE_ROLL.get_name().to_string()));
        o.insert("teamId".to_string(), jj::string_or_null(self.team_id.as_deref()));
        Ok(Value::Object(o))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_round_trip() {
        let p = DialogBriberyAndCorruptionParameter { team_id: Some("teamX".into()) };
        let json = serde_json::to_string(&p).unwrap();
        let back: DialogBriberyAndCorruptionParameter = serde_json::from_str(&json).unwrap();
        assert_eq!(back.get_team_id(), Some("teamX"));
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
        assert_eq!(round_trip(Rules::Bb2025, json!({"dialogId": "briberyAndCorruptionReRoll", "teamId": "t1"})).unwrap(), json!({"dialogId": "briberyAndCorruptionReRoll", "teamId": "t1"}));
    }
}
