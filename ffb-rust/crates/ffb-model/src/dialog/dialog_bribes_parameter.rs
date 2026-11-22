use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogBribesParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogBribesParameter {
    pub team_id: Option<String>,
    pub max_nr_of_bribes: i32,
    pub player_ids: Vec<String>,
}

impl DialogBribesParameter {
    pub fn new() -> Self { Self::default() }

    pub fn new_with(team_id: impl Into<String>, max_nr_of_bribes: i32) -> Self {
        DialogBribesParameter { team_id: Some(team_id.into()), max_nr_of_bribes, player_ids: Vec::new() }
    }

    pub fn get_team_id(&self) -> Option<&str> { self.team_id.as_deref() }
    pub fn get_max_nr_of_bribes(&self) -> i32 { self.max_nr_of_bribes }
    pub fn get_player_ids(&self) -> &[String] { &self.player_ids }

    pub fn add_player_id(&mut self, player_id: impl Into<String>) {
        let id = player_id.into();
        if !id.is_empty() { self.player_ids.push(id); }
    }
}

impl IDialogParameter for DialogBribesParameter {
    fn get_id(&self) -> DialogId { DialogId::BRIBES }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}

// JSON serialization (BACKLOG §O).
impl DialogBribesParameter {
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
        super::util_dialog_parameter::validate_dialog_id_json(DialogId::BRIBES, obj)?;
        self.team_id = jj::get_string(obj, "teamId")?;
        self.max_nr_of_bribes = jj::get_int(obj, "maxNrOfBribes")?;
        Self::add_provided(&mut self.player_ids, jj::get_string_array(obj, "playerIds")?);
        Ok(())
    }

    /// Java `toJsonValue()`.
    pub fn to_json_value(&self, _rules: crate::enums::Rules) -> Result<serde_json::Value, String> {
        use crate::util::java_json as jj;
        use serde_json::Value;
        let mut o = serde_json::Map::new();
        o.insert("dialogId".to_string(), Value::String(DialogId::BRIBES.get_name().to_string()));
        o.insert("teamId".to_string(), jj::string_or_null(self.team_id.as_deref()));
        o.insert("maxNrOfBribes".to_string(), Value::from(self.max_nr_of_bribes));
        o.insert("playerIds".to_string(), jj::string_array(&self.player_ids));
        Ok(Value::Object(o))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_player_id_appends_nonempty_strings() {
        let mut p = DialogBribesParameter::new_with("t", 2);
        p.add_player_id("p1");
        p.add_player_id("");
        assert_eq!(p.get_player_ids(), &["p1"]);
    }

    #[test]
    fn serde_round_trip() {
        let p = DialogBribesParameter::new_with("team2", 5);
        let json = serde_json::to_string(&p).unwrap();
        let back: DialogBribesParameter = serde_json::from_str(&json).unwrap();
        assert_eq!(back.get_max_nr_of_bribes(), 5);
        assert_eq!(back.get_team_id(), Some("team2"));
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
        assert_eq!(round_trip(Rules::Bb2025, json!({"dialogId": "bribes", "maxNrOfBribes": 1, "playerIds": ["p1"], "teamId": "t1"})).unwrap(), json!({"dialogId": "bribes", "maxNrOfBribes": 1, "playerIds": ["p1"], "teamId": "t1"}));
    }

    #[test]
    fn ids_are_filtered() {
        assert_eq!(round_trip(Rules::Bb2025, json!({"dialogId": "bribes", "playerIds": ["", null, "p2"]})).unwrap(), json!({"dialogId": "bribes", "maxNrOfBribes": 0, "playerIds": ["p2"], "teamId": null}));
    }
}
