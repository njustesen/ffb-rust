use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogSelectPositionParameter.
/// Note: PositionChoiceMode serialized as String (stub not yet translated).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogSelectPositionParameter {
    /// Java `List<String>` filled by `addAll`: a `None` is a null element.
    pub positions: Vec<Option<String>>,
    pub position_choice_mode: Option<String>,
    pub min_select: i32,
    pub max_select: i32,
    pub team_id: Option<String>,
}

impl DialogSelectPositionParameter {
    pub fn get_positions(&self) -> &[Option<String>] { &self.positions }
    pub fn get_position_choice_mode(&self) -> Option<&str> { self.position_choice_mode.as_deref() }
    pub fn get_min_select(&self) -> i32 { self.min_select }
    pub fn get_max_select(&self) -> i32 { self.max_select }
    pub fn get_team_id(&self) -> Option<&str> { self.team_id.as_deref() }
    pub fn add_position(&mut self, pos: impl Into<String>) {
        let s = pos.into();
        if !s.is_empty() { self.positions.push(Some(s)); }
    }

    /// Java: `transform()` returns `new DialogSelectPositionParameter(positions,
    /// positionChoiceMode, 1, 1, teamId)` — min/max select are hardcoded to 1, not
    /// carried over from the original instance.
    pub fn transform_typed(&self) -> Self {
        DialogSelectPositionParameter {
            positions: self.positions.clone(),
            position_choice_mode: self.position_choice_mode.clone(),
            min_select: 1,
            max_select: 1,
            team_id: self.team_id.clone(),
        }
    }
}

impl IDialogParameter for DialogSelectPositionParameter {
    fn get_id(&self) -> DialogId { DialogId::SELECT_POSITION }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.transform_typed()) }
}

// JSON serialization (BACKLOG §O). `position_choice_mode` holds the `PositionChoiceMode`
// constant name (what Java writes).
impl DialogSelectPositionParameter {
    /// Java `initFrom(IFactorySource, JsonValue)`.
    pub fn init_from(&mut self, _rules: crate::enums::Rules, json: &serde_json::Value) -> Result<(), String> {
        use crate::util::java_json as jj;
        let obj = jj::to_json_object(json)?;
        super::util_dialog_parameter::validate_dialog_id_json(DialogId::SELECT_POSITION, obj)?;
        // `Arrays.asList(array)` with no null check: absent -> NullPointerException; null and ""
        // elements are kept as they are.
        let position_ids = jj::get_string_array(obj, "positionIds")?
            .ok_or_else(|| "positionIds: null (NullPointerException)".to_string())?;
        self.positions.extend(position_ids);
        // PositionChoiceMode.valueOf: the exact constant name (null / unknown throw).
        let mode = crate::model::position_choice_mode::PositionChoiceMode::value_of(
            jj::get_string(obj, "positionChoiceMode")?.as_deref(),
        )?;
        self.position_choice_mode = Some(mode.name().to_string());
        self.min_select = jj::get_int(obj, "minSelects")?;
        self.max_select = jj::get_int(obj, "maxSelects")?;
        self.team_id = jj::get_string(obj, "teamId")?;
        Ok(())
    }

    /// Java `toJsonValue()`.
    pub fn to_json_value(&self, _rules: crate::enums::Rules) -> Result<serde_json::Value, String> {
        use crate::util::java_json as jj;
        use serde_json::Value;
        let mut o = serde_json::Map::new();
        o.insert("dialogId".to_string(), Value::String(DialogId::SELECT_POSITION.get_name().to_string()));
        o.insert("positionIds".to_string(), jj::string_array_or_null(Some(self.positions.as_slice())));
        // `positionChoiceMode.name()`: a null mode -> NullPointerException.
        let mode = self.position_choice_mode.as_deref().ok_or_else(|| {
            "DialogSelectPositionParameter.toJsonValue: positionChoiceMode is null (NullPointerException)".to_string()
        })?;
        o.insert("positionChoiceMode".to_string(), Value::String(mode.to_string()));
        o.insert("minSelects".to_string(), Value::from(self.min_select));
        o.insert("maxSelects".to_string(), Value::from(self.max_select));
        o.insert("teamId".to_string(), jj::string_or_null(self.team_id.as_deref()));
        Ok(Value::Object(o))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn add_position_filters_empty_string() {
        let mut p = DialogSelectPositionParameter::default();
        p.add_position("pos1");
        p.add_position("");
        assert_eq!(p.get_positions().len(), 1);
    }

    #[test]
    fn transform_resets_min_max_select_to_one() {
        // Java's transform() always hardcodes minSelect/maxSelect to 1, regardless of
        // the original values — it does NOT just clone the instance.
        let p = DialogSelectPositionParameter {
            positions: vec![Some("Blitzer".into())],
            position_choice_mode: Some("MULTIPLE".into()),
            min_select: 2,
            max_select: 5,
            team_id: Some("home".into()),
        };
        let transformed = p.transform_typed();
        assert_eq!(transformed.get_min_select(), 1);
        assert_eq!(transformed.get_max_select(), 1);
        assert_eq!(transformed.get_team_id(), Some("home"));
        assert_eq!(transformed.get_positions(), &[Some("Blitzer".to_string())]);
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
        assert_eq!(round_trip(Rules::Bb2025, json!({"dialogId": "selectPosition", "maxSelects": 1, "minSelects": 1, "positionChoiceMode": "RAISE_DEAD", "positionIds": ["66211", "66210"], "teamId": "t1"})).unwrap(), json!({"dialogId": "selectPosition", "maxSelects": 1, "minSelects": 1, "positionChoiceMode": "RAISE_DEAD", "positionIds": ["66211", "66210"], "teamId": "t1"}));
    }

    #[test]
    fn null_and_empty_ids_are_kept() {
        assert_eq!(round_trip(Rules::Bb2025, json!({"dialogId": "selectPosition", "positionChoiceMode": "RAISE_DEAD", "positionIds": ["", null]})).unwrap(), json!({"dialogId": "selectPosition", "maxSelects": 0, "minSelects": 0, "positionChoiceMode": "RAISE_DEAD", "positionIds": ["", null], "teamId": null}));
    }

    #[test]
    fn mode_name_is_exact() {
        assert!(round_trip(Rules::Bb2025, json!({"dialogId": "selectPosition", "positionChoiceMode": "raiseDead", "positionIds": []})).is_err());
    }

    #[test]
    fn missing_ids_throws() {
        assert!(round_trip(Rules::Bb2025, json!({"dialogId": "selectPosition", "positionChoiceMode": "RAISE_DEAD"})).is_err());
    }
}
