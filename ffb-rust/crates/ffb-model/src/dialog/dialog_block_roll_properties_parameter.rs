use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::enums::ReRollProperty;
use super::dialog_id::DialogId;
use super::has_re_roll_properties::HasReRollProperties;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogBlockRollPropertiesParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogBlockRollPropertiesParameter {
    pub choosing_team_id: Option<String>,
    pub nr_of_dice: i32,
    pub block_roll: Vec<i32>,
    /// Java: `blockRoll` is null (JSON `blockRoll` absent or null).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub block_roll_null: bool,
    /// Java `final List<ReRollProperty>`; a decoded unknown name is a null entry (`None`).
    pub re_roll_properties: Vec<Option<ReRollProperty>>,
    /// Maps re-rolled action name to source name (Java `Map<String, String>`; a value the JSON
    /// left empty or null is a null value).
    pub rr_action_to_source: HashMap<String, Option<String>>,
}

impl DialogBlockRollPropertiesParameter {
    pub fn get_choosing_team_id(&self) -> Option<&str> { self.choosing_team_id.as_deref() }
    pub fn get_nr_of_dice(&self) -> i32 { self.nr_of_dice }
    pub fn get_block_roll(&self) -> &[i32] { &self.block_roll }
    pub fn get_re_roll_properties(&self) -> &[Option<ReRollProperty>] { &self.re_roll_properties }
    pub fn get_rr_action_to_source(&self) -> &HashMap<String, Option<String>> { &self.rr_action_to_source }

    /// Java: hasActualReRoll().
    pub fn has_actual_re_roll(&self) -> bool {
        self.re_roll_properties.iter().flatten().any(|p| p.is_actual_reroll()) || !self.rr_action_to_source.is_empty()
    }
}

impl HasReRollProperties for DialogBlockRollPropertiesParameter {
    fn has_property(&self, property: ReRollProperty) -> bool {
        self.re_roll_properties.contains(&Some(property))
    }
}

impl IDialogParameter for DialogBlockRollPropertiesParameter {
    fn get_id(&self) -> DialogId { DialogId::BLOCK_ROLL_PROPERTIES }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}

// JSON serialization (BACKLOG §O).
impl DialogBlockRollPropertiesParameter {
    /// Java `initFrom(IFactorySource, JsonValue)`.
    pub fn init_from(&mut self, rules: crate::enums::Rules, json: &serde_json::Value) -> Result<(), String> {
        use crate::util::java_json as jj;
        let obj = jj::to_json_object(json)?;
        super::util_dialog_parameter::validate_dialog_id_json(DialogId::BLOCK_ROLL_PROPERTIES, obj)?;
        self.choosing_team_id = jj::get_string(obj, "choosingTeamId")?;
        self.nr_of_dice = jj::get_int(obj, "nrOfDice")?;
        let block_roll = jj::get_int_array(obj, "blockRoll")?;
        self.block_roll_null = block_roll.is_none();
        self.block_roll = block_roll.unwrap_or_default();
        self.re_roll_properties.extend(ReRollProperty::list_from_json(rules, obj)?);
        self.rr_action_to_source.extend(jj::get_string_map(obj, "reRollActionToSourceMap")?);
        Ok(())
    }

    /// Java `toJsonValue()`.
    pub fn to_json_value(&self, _rules: crate::enums::Rules) -> Result<serde_json::Value, String> {
        use crate::util::java_json as jj;
        use serde_json::Value;
        let mut o = serde_json::Map::new();
        o.insert("dialogId".into(), Value::from(DialogId::BLOCK_ROLL_PROPERTIES.get_name()));
        o.insert("choosingTeamId".into(), jj::string_or_null(self.choosing_team_id.as_deref()));
        o.insert("nrOfDice".into(), Value::from(self.nr_of_dice));
        o.insert("blockRoll".into(), jj::int_array_or_null((!self.block_roll_null).then_some(self.block_roll.as_slice())));
        o.insert("reRollProperties".into(), ReRollProperty::list_to_json(&self.re_roll_properties)?);
        let map: Vec<(String, Option<String>)> = self.rr_action_to_source.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
        o.insert("reRollActionToSourceMap".into(), jj::string_map(&map));
        Ok(Value::Object(o))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn has_property_true_when_present() {
        let p = DialogBlockRollPropertiesParameter {
            re_roll_properties: vec![Some(ReRollProperty::Trr)],
            ..Default::default()
        };
        assert!(p.has_property(ReRollProperty::Trr));
        assert!(!p.has_property(ReRollProperty::Loner));
    }

    #[test]
    fn has_actual_re_roll_true_for_trr() {
        let p = DialogBlockRollPropertiesParameter {
            re_roll_properties: vec![Some(ReRollProperty::Trr)],
            ..Default::default()
        };
        assert!(p.has_actual_re_roll());
    }

    #[test]
    fn has_actual_re_roll_false_for_loner_only() {
        let p = DialogBlockRollPropertiesParameter {
            re_roll_properties: vec![Some(ReRollProperty::Loner)],
            ..Default::default()
        };
        assert!(!p.has_actual_re_roll());
    }

    #[test]
    fn has_actual_re_roll_true_when_rr_action_to_source_nonempty() {
        let p = DialogBlockRollPropertiesParameter {
            rr_action_to_source: [("block".to_string(), Some("team".to_string()))].into(),
            ..Default::default()
        };
        assert!(p.has_actual_re_roll());
    }

    #[test]
    fn has_actual_re_roll_false_when_empty() {
        let p = DialogBlockRollPropertiesParameter::default();
        assert!(!p.has_actual_re_roll());
    }

    fn round_trip(rules: crate::enums::Rules, input: serde_json::Value) -> Result<serde_json::Value, String> {
        let mut p = DialogBlockRollPropertiesParameter::default();
        p.init_from(rules, &input)?;
        p.to_json_value(rules)
    }

    #[test]
    fn json_round_trip_matches_java() {
        use crate::enums::Rules;
        use serde_json::json;
        let input = json!({"dialogId": "blockRollProperties", "choosingTeamId": "t", "nrOfDice": 2, "blockRoll": [2, 4],
            "reRollProperties": ["TRR", "PRO"], "reRollActionToSourceMap": {"Single Die Per Activation": "Pro"}});
        assert_eq!(round_trip(Rules::Bb2025, input.clone()).unwrap(), input);
        // An empty map value is read as null and written as null; a non-object map is empty.
        let out = round_trip(Rules::Bb2025, json!({"dialogId": "blockRollProperties", "reRollProperties": [],
            "reRollActionToSourceMap": {"a": ""}})).unwrap();
        assert_eq!(out["reRollActionToSourceMap"], json!({"a": null}));
        assert_eq!(out["blockRoll"], json!(null));
        let out = round_trip(Rules::Bb2025, json!({"dialogId": "blockRollProperties", "reRollProperties": [],
            "reRollActionToSourceMap": 3})).unwrap();
        assert_eq!(out["reRollActionToSourceMap"], json!({}));
        // reRollProperties is required and BB2025-only.
        assert!(round_trip(Rules::Bb2025, json!({"dialogId": "blockRollProperties"})).is_err());
        assert!(round_trip(Rules::Bb2020, json!({"dialogId": "blockRollProperties", "reRollProperties": []})).is_err());
    }
}
