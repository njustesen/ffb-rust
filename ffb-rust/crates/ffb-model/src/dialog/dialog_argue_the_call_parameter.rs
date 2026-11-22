use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogArgueTheCallParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogArgueTheCallParameter {
    pub team_id: Option<String>,
    pub player_ids: Vec<String>,
    pub stay_on_pitch: bool,
    pub friends_with_the_ref: bool,
    pub biased_refs: i32,
}

impl DialogArgueTheCallParameter {
    pub fn get_team_id(&self) -> Option<&str> { self.team_id.as_deref() }
    pub fn get_player_ids(&self) -> &[String] { &self.player_ids }
    pub fn is_stay_on_pitch(&self) -> bool { self.stay_on_pitch }
    pub fn is_friends_with_the_ref(&self) -> bool { self.friends_with_the_ref }
    pub fn get_biased_refs(&self) -> i32 { self.biased_refs }
    pub fn add_player_id(&mut self, id: impl Into<String>) {
        let s = id.into();
        if !s.is_empty() { self.player_ids.push(s); }
    }
}

impl IDialogParameter for DialogArgueTheCallParameter {
    fn get_id(&self) -> DialogId { DialogId::ARGUE_THE_CALL }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}

// JSON serialization (BACKLOG §O).
impl DialogArgueTheCallParameter {
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
        super::util_dialog_parameter::validate_dialog_id_json(DialogId::ARGUE_THE_CALL, obj)?;
        self.team_id = jj::get_string(obj, "teamId")?;
        Self::add_provided(&mut self.player_ids, jj::get_string_array(obj, "playerIds")?);
        // `Boolean value = getFrom(..); field = value != null && value` (JSON null still throws).
        self.stay_on_pitch = jj::get_boolean_object(obj, "staysOnPitch")?.unwrap_or(false);
        self.friends_with_the_ref = jj::get_boolean_object(obj, "friendsWithRef")?.unwrap_or(false);
        self.biased_refs = jj::get_int(obj, "biasedRefs")?;
        Ok(())
    }

    /// Java `toJsonValue()`.
    pub fn to_json_value(&self, _rules: crate::enums::Rules) -> Result<serde_json::Value, String> {
        use crate::util::java_json as jj;
        use serde_json::Value;
        let mut o = serde_json::Map::new();
        o.insert("dialogId".to_string(), Value::String(DialogId::ARGUE_THE_CALL.get_name().to_string()));
        o.insert("teamId".to_string(), jj::string_or_null(self.team_id.as_deref()));
        o.insert("playerIds".to_string(), jj::string_array(&self.player_ids));
        o.insert("staysOnPitch".to_string(), Value::Bool(self.stay_on_pitch));
        o.insert("friendsWithRef".to_string(), Value::Bool(self.friends_with_the_ref));
        o.insert("biasedRefs".to_string(), Value::from(self.biased_refs));
        Ok(Value::Object(o))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_player_id_appends_nonempty_strings() {
        let mut p = DialogArgueTheCallParameter::default();
        p.add_player_id("p1");
        p.add_player_id("p2");
        assert_eq!(p.get_player_ids(), &["p1", "p2"]);
    }

    #[test]
    fn add_player_id_ignores_empty_string() {
        let mut p = DialogArgueTheCallParameter::default();
        p.add_player_id("");
        assert!(p.get_player_ids().is_empty());
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
        assert_eq!(round_trip(Rules::Bb2025, json!({"biasedRefs": 0, "dialogId": "argueTheCall", "friendsWithRef": false, "playerIds": ["p1"], "staysOnPitch": true, "teamId": "t1"})).unwrap(), json!({"biasedRefs": 0, "dialogId": "argueTheCall", "friendsWithRef": false, "playerIds": ["p1"], "staysOnPitch": true, "teamId": "t1"}));
    }

    #[test]
    fn missing_booleans_are_false() {
        assert_eq!(round_trip(Rules::Bb2016, json!({"dialogId": "argueTheCall"})).unwrap(), json!({"biasedRefs": 0, "dialogId": "argueTheCall", "friendsWithRef": false, "playerIds": [], "staysOnPitch": false, "teamId": null}));
    }

    #[test]
    fn null_boolean_throws() {
        assert!(round_trip(Rules::Bb2016, json!({"dialogId": "argueTheCall", "staysOnPitch": null})).is_err());
    }
}
