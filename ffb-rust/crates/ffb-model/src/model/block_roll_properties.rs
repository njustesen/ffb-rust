use serde::{Deserialize, Serialize};
use crate::enums::{PlayerState, ReRollProperty, Rules};

/// 1:1 translation of com.fumbbl.ffb.model.BlockRollProperties.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockRollProperties {
    pub target_id: Option<String>,
    pub old_player_state: Option<PlayerState>,
    pub successful_dauntless: bool,
    pub own_choice: bool,
    pub double_target_strength: bool,
    pub nr_of_dice: i32,
    pub id: i32,
    pub pro_index: i32,
    /// Java `int[] blockRoll` (null until set).
    pub block_roll: Option<Vec<i32>>,
    /// Java `int[] reRollDiceIndexes = new int[0]` (null only when JSON says so).
    pub re_roll_dice_indexes: Option<Vec<i32>>,
    pub selected_index: i32,
    /// Java `final Map<String, String> rrActionToSource` (a HashMap; insertion order here).
    pub rr_action_to_source: Vec<(String, Option<String>)>,
    /// Java `final Set<ReRollProperty> reRollProperties` (a HashSet: no duplicates; a decoded
    /// unknown name is a null element, `None`).
    pub re_roll_properties: Vec<Option<ReRollProperty>>,
}

impl Default for BlockRollProperties {
    fn default() -> Self {
        BlockRollProperties {
            target_id: None,
            old_player_state: None,
            successful_dauntless: false,
            own_choice: false,
            double_target_strength: false,
            nr_of_dice: 0,
            id: 0,
            pro_index: 0,
            block_roll: None,
            re_roll_dice_indexes: Some(Vec::new()),
            selected_index: -1,
            rr_action_to_source: Vec::new(),
            re_roll_properties: Vec::new(),
        }
    }
}

impl BlockRollProperties {
    pub fn new() -> Self { BlockRollProperties::default() }

    pub fn new_with(target_id: impl Into<String>, old_player_state: Option<PlayerState>, id: i32) -> Self {
        BlockRollProperties { target_id: Some(target_id.into()), old_player_state, id, ..Default::default() }
    }

    pub fn get_id(&self) -> i32 { self.id }
    pub fn get_target_id(&self) -> Option<&str> { self.target_id.as_deref() }
    pub fn needs_selection(&self) -> bool { self.selected_index < 0 }

    /// Java `add(ReRollProperty)` = `HashSet.add`.
    pub fn add(&mut self, property: ReRollProperty) {
        if !self.re_roll_properties.contains(&Some(property)) {
            self.re_roll_properties.push(Some(property));
        }
    }

    pub fn has_property(&self, property: ReRollProperty) -> bool {
        self.re_roll_properties.contains(&Some(property))
    }

    /// Java `initFrom(IFactorySource, JsonValue)`.
    pub fn init_from(&mut self, rules: Rules, json: &serde_json::Value) -> Result<(), String> {
        use crate::util::java_json as jj;
        let obj = jj::to_json_object(json)?;
        self.target_id = jj::get_string(obj, "playerId")?;
        self.successful_dauntless = jj::get_boolean(obj, "successfulDauntless")?;
        self.nr_of_dice = jj::get_int(obj, "nrOfDice")?;
        self.block_roll = jj::get_int_array(obj, "blockRoll")?;
        self.selected_index = jj::get_int(obj, "selectedIndex")?;
        self.own_choice = jj::get_boolean(obj, "isOwnChoice")?;
        self.old_player_state = jj::get_player_state(obj, "playerStateOld")?;
        self.id = jj::get_int(obj, "blockRollId")?;
        // Map.putAll: an existing key takes the new value in place.
        for (k, v) in jj::get_string_map(obj, "reRollActionToSourceMap")? {
            match self.rr_action_to_source.iter_mut().find(|(n, _)| *n == k) {
                Some(e) => e.1 = v,
                None => self.rr_action_to_source.push((k, v)),
            }
        }
        self.re_roll_dice_indexes = jj::get_int_array(obj, "reRolledDice")?;
        self.pro_index = jj::get_int(obj, "proIndex")?;
        // toPrimitive(Boolean): null -> false.
        self.double_target_strength = jj::get_boolean_object(obj, "doubleTargetStrength")?.unwrap_or(false);
        // Set.addAll: duplicates (incl. two nulls) collapse.
        for p in ReRollProperty::list_from_json(rules, obj)? {
            if !self.re_roll_properties.contains(&p) {
                self.re_roll_properties.push(p);
            }
        }
        Ok(())
    }

    /// Java `toJsonValue()`.
    pub fn to_json_value(&self) -> Result<serde_json::Value, String> {
        use crate::util::java_json as jj;
        use serde_json::Value;
        // reRollProperties is a HashSet of enum constants: with two or more elements Java's
        // iteration order is the identity-hash order of one JVM run (not reproducible, and the
        // path is not in the canonical form's unordered list). None occur in the corpus.
        if self.re_roll_properties.len() > 1 {
            return Err(format!("BlockRollProperties: {} reRollProperties have no reproducible Java order", self.re_roll_properties.len()));
        }
        let mut o = serde_json::Map::new();
        o.insert("playerId".into(), jj::string_or_null(self.target_id.as_deref()));
        o.insert("successfulDauntless".into(), Value::Bool(self.successful_dauntless));
        o.insert("nrOfDice".into(), Value::from(self.nr_of_dice));
        o.insert("blockRoll".into(), jj::int_array_or_null(self.block_roll.as_deref()));
        o.insert("selectedIndex".into(), Value::from(self.selected_index));
        o.insert("isOwnChoice".into(), Value::Bool(self.own_choice));
        o.insert("playerStateOld".into(), jj::player_state_or_null(self.old_player_state.map(|s| s.0 as i32)));
        o.insert("blockRollId".into(), Value::from(self.id));
        o.insert("reRollActionToSourceMap".into(), jj::string_map(&self.rr_action_to_source));
        o.insert("reRolledDice".into(), jj::int_array_or_null(self.re_roll_dice_indexes.as_deref()));
        o.insert("proIndex".into(), Value::from(self.pro_index));
        o.insert("doubleTargetStrength".into(), Value::Bool(self.double_target_strength));
        o.insert("reRollProperties".into(), ReRollProperty::list_to_json(&self.re_roll_properties)?);
        Ok(Value::Object(o))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn defaults_match_java_field_initialisers() {
        let p = BlockRollProperties::new();
        assert_eq!(p.selected_index, -1);
        assert_eq!(p.re_roll_dice_indexes, Some(vec![]));
        assert!(p.needs_selection());
    }

    #[test]
    fn json_round_trip_matches_java() {
        let input = json!({"blockRoll":[5,2],"blockRollId":1,"doubleTargetStrength":false,"isOwnChoice":true,
            "nrOfDice":2,"playerId":"17638746","playerStateOld":257,"proIndex":0,"reRollActionToSourceMap":{},
            "reRollProperties":[],"reRolledDice":[],"selectedIndex":0,"successfulDauntless":false});
        let mut p = BlockRollProperties::new();
        p.init_from(Rules::Bb2025, &input).unwrap();
        assert_eq!(p.to_json_value().unwrap(), input);
    }

    #[test]
    fn json_absent_members_follow_the_option_types() {
        // selectedIndex absent -> JsonIntOption 0 (not the field's -1); doubleTargetStrength
        // absent -> false; reRolledDice absent -> null; successfulDauntless absent -> NPE.
        let mut p = BlockRollProperties::new();
        p.init_from(Rules::Bb2025, &json!({"successfulDauntless": true, "isOwnChoice": false, "reRollProperties": ["trr", "TRR"]})).unwrap();
        let out = p.to_json_value().unwrap();
        assert_eq!(out["selectedIndex"], json!(0));
        assert_eq!(out["doubleTargetStrength"], json!(false));
        assert_eq!(out["reRolledDice"], json!(null));
        assert_eq!(out["reRollProperties"], json!(["TRR"]));
        assert!(BlockRollProperties::new().init_from(Rules::Bb2025, &json!({"isOwnChoice": false, "reRollProperties": []})).is_err());
        // No ReRollPropertyFactory outside BB2025.
        assert!(BlockRollProperties::new().init_from(Rules::Bb2020, &json!({"successfulDauntless": true, "isOwnChoice": false, "reRollProperties": []})).is_err());
    }
}
