use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogWizardSpellParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogWizardSpellParameter {
    pub team_id: Option<String>,
}

impl DialogWizardSpellParameter {
    pub fn get_team_id(&self) -> Option<&str> { self.team_id.as_deref() }
}

impl IDialogParameter for DialogWizardSpellParameter {
    fn get_id(&self) -> DialogId { DialogId::WIZARD_SPELL }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}

// JSON serialization (BACKLOG §O). Java extends `DialogWithoutParameter` but overrides `toJsonValue` with
// `{dialogId, teamId}`.
impl DialogWizardSpellParameter {
    /// Java `initFrom(IFactorySource, JsonValue)`.
    pub fn init_from(&mut self, _rules: crate::enums::Rules, json: &serde_json::Value) -> Result<(), String> {
        use crate::util::java_json as jj;
        let obj = jj::to_json_object(json)?;
        super::util_dialog_parameter::validate_dialog_id_json(DialogId::WIZARD_SPELL, obj)?;
        self.team_id = jj::get_string(obj, "teamId")?;
        Ok(())
    }

    /// Java `toJsonValue()`.
    pub fn to_json_value(&self, _rules: crate::enums::Rules) -> Result<serde_json::Value, String> {
        use crate::util::java_json as jj;
        use serde_json::Value;
        let mut o = serde_json::Map::new();
        o.insert("dialogId".to_string(), Value::String(DialogId::WIZARD_SPELL.get_name().to_string()));
        o.insert("teamId".to_string(), jj::string_or_null(self.team_id.as_deref()));
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
        assert_eq!(round_trip(Rules::Bb2025, json!({"dialogId": "wizardSpell", "teamId": "t1"})).unwrap(), json!({"dialogId": "wizardSpell", "teamId": "t1"}));
    }
}
