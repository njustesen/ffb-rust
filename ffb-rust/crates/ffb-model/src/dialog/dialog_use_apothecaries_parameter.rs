use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogUseApothecariesParameter.
/// The `InjuryDescription`s are kept as their Java JSON (`InjuryDescription.toJsonValue()`), the
/// form every user of this parameter consumes.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogUseApothecariesParameter {
    pub team_id: Option<String>,
    pub injury_descriptions: Vec<serde_json::Value>,
}

impl DialogUseApothecariesParameter {
    pub fn get_team_id(&self) -> Option<&str> { self.team_id.as_deref() }
    pub fn get_injury_descriptions(&self) -> &[serde_json::Value] { &self.injury_descriptions }
}

impl IDialogParameter for DialogUseApothecariesParameter {
    fn get_id(&self) -> DialogId { DialogId::USE_APOTHECARIES }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}

// JSON serialization (BACKLOG §O).
impl DialogUseApothecariesParameter {
    /// Java `initFrom(IFactorySource, JsonValue)`.
    pub fn init_from(&mut self, rules: crate::enums::Rules, json: &serde_json::Value) -> Result<(), String> {
        use crate::bb2020::injury_description::InjuryDescription;
        use crate::util::java_json as jj;
        let obj = jj::to_json_object(json)?;
        super::util_dialog_parameter::validate_dialog_id_json(DialogId::USE_APOTHECARIES, obj)?;
        // JsonArrayOption never returns null (absent/null -> empty), so the Java null check is moot.
        for value in jj::get_json_array_option(obj, "injuryDescriptions")? {
            let mut description = InjuryDescription::new();
            description.init_from(rules, &value)?;
            self.injury_descriptions.push(description.to_json_value());
        }
        self.team_id = jj::get_string(obj, "teamId")?;
        Ok(())
    }

    /// Java `toJsonValue()`.
    pub fn to_json_value(&self, _rules: crate::enums::Rules) -> Result<serde_json::Value, String> {
        use crate::util::java_json as jj;
        use serde_json::Value;
        let mut o = serde_json::Map::new();
        o.insert("dialogId".into(), Value::from(DialogId::USE_APOTHECARIES.get_name()));
        o.insert("injuryDescriptions".into(), Value::Array(self.injury_descriptions.clone()));
        o.insert("teamId".into(), jj::string_or_null(self.team_id.as_deref()));
        Ok(Value::Object(o))
    }
}

#[cfg(test)]
mod tests {
    use crate::dialog::dialog_parameter_factory::DialogParameterFactory;
    use crate::enums::Rules;
    use serde_json::json;

    #[test]
    fn json_round_trip_matches_java() {
        let input = json!({"dialogId":"useApothecaries","injuryDescriptions":[{"apothecaryTypes":["TEAM"],
            "playerId":"p1","playerState":5,"seriousInjury":null}],"teamId":"t1"});
        let p = DialogParameterFactory::new().for_json_value(Rules::Bb2025, &input).unwrap().unwrap();
        assert_eq!(p.to_json_value(Rules::Bb2025).unwrap(), input);
        // An unknown injury name decodes to null; an unknown apothecary type throws.
        let input = json!({"dialogId":"useApothecaries","injuryDescriptions":[{"playerId":"p1","seriousInjury":"Broken Shoulder (-ST)"}]});
        let p = DialogParameterFactory::new().for_json_value(Rules::Bb2025, &input).unwrap().unwrap();
        assert_eq!(p.to_json_value(Rules::Bb2025).unwrap(), json!({"dialogId":"useApothecaries","injuryDescriptions":[
            {"apothecaryTypes":[],"playerId":"p1","playerState":null,"seriousInjury":null}],"teamId":null}));
        let bad = json!({"dialogId":"useApothecaries","injuryDescriptions":[{"apothecaryTypes":["IGOR"]}]});
        assert!(DialogParameterFactory::new().for_json_value(Rules::Bb2025, &bad).is_err());
    }
}
