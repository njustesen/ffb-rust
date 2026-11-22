use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogPlayerChoiceParameter.
/// Note: PlayerChoiceMode serialized as String name (stub not yet translated).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogPlayerChoiceParameter {
    pub team_id: Option<String>,
    /// PlayerChoiceMode serialized by name.
    pub player_choice_mode: Option<String>,
    pub player_ids: Vec<String>,
    pub descriptions: Vec<String>,
    pub max_selects: i32,
    pub min_selects: i32,
}

impl DialogPlayerChoiceParameter {
    pub fn get_team_id(&self) -> Option<&str> { self.team_id.as_deref() }
    pub fn get_player_choice_mode(&self) -> Option<&str> { self.player_choice_mode.as_deref() }
    pub fn get_player_ids(&self) -> &[String] { &self.player_ids }
    pub fn get_descriptions(&self) -> &[String] { &self.descriptions }
    pub fn get_max_selects(&self) -> i32 { self.max_selects }
    pub fn get_min_selects(&self) -> i32 { self.min_selects }
    pub fn add_player_id(&mut self, id: impl Into<String>) {
        let s = id.into();
        if !s.is_empty() { self.player_ids.push(s); }
    }
    pub fn add_description(&mut self, d: impl Into<String>) {
        let s = d.into();
        if !s.is_empty() { self.descriptions.push(s); }
    }
}

impl IDialogParameter for DialogPlayerChoiceParameter {
    fn get_id(&self) -> DialogId { DialogId::PLAYER_CHOICE }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}

// JSON serialization (BACKLOG §O).
impl DialogPlayerChoiceParameter {
    /// Java `addXxx(String[])`: `ArrayTool.isProvided` (a null or empty array adds nothing), each
    /// element through the `StringTool.isProvided` filter (null and "" are skipped).
    fn add_provided(target: &mut Vec<String>, values: Option<Vec<Option<String>>>) {
        for value in values.into_iter().flatten().flatten() {
            if crate::util::java_json::is_provided(Some(&value)) {
                target.push(value);
            }
        }
    }

    /// Java `initFrom(IFactorySource, JsonValue)`.
    pub fn init_from(&mut self, _rules: crate::enums::Rules, json: &serde_json::Value) -> Result<(), String> {
        use crate::util::java_json as jj;
        let obj = jj::to_json_object(json)?;
        super::util_dialog_parameter::validate_dialog_id_json(DialogId::PLAYER_CHOICE, obj)?;
        self.team_id = jj::get_string(obj, "teamId")?;
        // JsonEnumWithNameOption over PlayerChoiceModeFactory: equalsIgnoreCase, unknown -> null
        // (e.g. a newer server's mode), which toJsonValue then writes as null.
        self.player_choice_mode = jj::get_enum_name(obj, "playerChoiceMode")?
            .and_then(crate::model::player_choice_mode::PlayerChoiceMode::for_name_ignore_case)
            .map(|mode| mode.get_name().to_string());
        self.max_selects = jj::get_int(obj, "maxSelects")?;
        Self::add_provided(&mut self.player_ids, jj::get_string_array(obj, "playerIds")?);
        Self::add_provided(&mut self.descriptions, jj::get_string_array(obj, "descriptions")?);
        self.min_selects = jj::get_int(obj, "minSelects")?;
        Ok(())
    }

    /// Java `toJsonValue()`.
    pub fn to_json_value(&self, _rules: crate::enums::Rules) -> Result<serde_json::Value, String> {
        use crate::util::java_json as jj;
        use serde_json::Value;
        let mut o = serde_json::Map::new();
        o.insert("dialogId".to_string(), Value::String(DialogId::PLAYER_CHOICE.get_name().to_string()));
        o.insert("teamId".to_string(), jj::string_or_null(self.team_id.as_deref()));
        o.insert("playerChoiceMode".to_string(), jj::string_or_null(self.player_choice_mode.as_deref()));
        o.insert("maxSelects".to_string(), Value::from(self.max_selects));
        o.insert("playerIds".to_string(), jj::string_array(&self.player_ids));
        o.insert("descriptions".to_string(), jj::string_array(&self.descriptions));
        o.insert("minSelects".to_string(), Value::from(self.min_selects));
        Ok(Value::Object(o))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_player_id_appends_nonempty() {
        let mut p = DialogPlayerChoiceParameter::default();
        p.add_player_id("p1");
        p.add_player_id("");
        assert_eq!(p.get_player_ids(), &["p1"]);
    }

    #[test]
    fn add_description_appends_nonempty() {
        let mut p = DialogPlayerChoiceParameter::default();
        p.add_description("Select a target");
        p.add_description("");
        assert_eq!(p.get_descriptions(), &["Select a target"]);
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
    fn round_trip_mvp() {
        assert_eq!(round_trip(Rules::Bb2025, json!({"dialogId": "playerChoice", "teamId": "t1", "playerChoiceMode": "mvp", "maxSelects": 6, "minSelects": 6, "playerIds": ["p1", "p2"], "descriptions": []})).unwrap(), json!({"dialogId": "playerChoice", "teamId": "t1", "playerChoiceMode": "mvp", "maxSelects": 6, "minSelects": 6, "playerIds": ["p1", "p2"], "descriptions": []}));
    }

    #[test]
    fn unknown_mode_is_null_and_ids_are_filtered() {
        assert_eq!(round_trip(Rules::Bb2025, json!({"dialogId": "playerChoice", "teamId": "t1", "playerChoiceMode": "dwarfenWisdom", "playerIds": ["p1", "", null], "descriptions": ["x", ""]})).unwrap(), json!({"dialogId": "playerChoice", "teamId": "t1", "playerChoiceMode": null, "maxSelects": 0, "minSelects": 0, "playerIds": ["p1"], "descriptions": ["x"]}));
    }

    #[test]
    fn mode_is_case_insensitive_and_canonical() {
        assert_eq!(round_trip(Rules::Bb2016, json!({"dialogId": "playerChoice", "playerChoiceMode": "MVP"})).unwrap(), json!({"dialogId": "playerChoice", "teamId": null, "playerChoiceMode": "mvp", "maxSelects": 0, "minSelects": 0, "playerIds": [], "descriptions": []}));
    }
}
