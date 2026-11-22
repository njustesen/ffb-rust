use serde::{Deserialize, Serialize};
use crate::enums::ReRollSource;
use crate::enums::SkillId;
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogBlockRollPartialReRollParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogBlockRollPartialReRollParameter {
    pub choosing_team_id: Option<String>,
    pub nr_of_dice: i32,
    pub block_roll: Vec<i32>,
    /// Java: `fBlockRoll` is null (JSON `blockRoll` absent or null).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub block_roll_null: bool,
    pub re_rolled_dice_indexes: Vec<i32>,
    /// Java: `reRolledDiceIndexes` is null (JSON `reRolledDice` absent or null).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub re_rolled_dice_indexes_null: bool,
    pub team_re_roll_option: bool,
    pub pro_re_roll_option: bool,
    pub brawler_option: bool,
    pub consummate_option: bool,
    pub single_use_re_roll_source: Option<ReRollSource>,
    /// Java `List<Skill>`; a decoded unknown skill name is a null entry (`None`).
    pub re_roll_explicit_die_skills: Vec<Option<SkillId>>,
}

impl DialogBlockRollPartialReRollParameter {
    pub fn get_choosing_team_id(&self) -> Option<&str> { self.choosing_team_id.as_deref() }
    pub fn get_nr_of_dice(&self) -> i32 { self.nr_of_dice }
    pub fn get_block_roll(&self) -> &[i32] { &self.block_roll }
    pub fn get_re_rolled_dice_indexes(&self) -> &[i32] { &self.re_rolled_dice_indexes }
    pub fn has_team_re_roll_option(&self) -> bool { self.team_re_roll_option }
    pub fn has_pro_re_roll_option(&self) -> bool { self.pro_re_roll_option }
    pub fn has_brawler_option(&self) -> bool { self.brawler_option }
    pub fn has_consummate_option(&self) -> bool { self.consummate_option }
}

impl IDialogParameter for DialogBlockRollPartialReRollParameter {
    fn get_id(&self) -> DialogId { DialogId::BLOCK_ROLL_PARTIAL_RE_ROLL }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}

// JSON serialization (BACKLOG §O).
impl DialogBlockRollPartialReRollParameter {
    /// Java `initFrom(IFactorySource, JsonValue)`.
    pub fn init_from(&mut self, rules: crate::enums::Rules, json: &serde_json::Value) -> Result<(), String> {
        use crate::factory::re_roll_source_factory::ReRollSourceFactory;
        use crate::factory::skill_factory::SkillFactory;
        use crate::util::java_json as jj;
        let obj = jj::to_json_object(json)?;
        super::util_dialog_parameter::validate_dialog_id_json(DialogId::BLOCK_ROLL_PARTIAL_RE_ROLL, obj)?;
        self.choosing_team_id = jj::get_string(obj, "choosingTeamId")?;
        self.nr_of_dice = jj::get_int(obj, "nrOfDice")?;
        let block_roll = jj::get_int_array(obj, "blockRoll")?;
        self.block_roll_null = block_roll.is_none();
        self.block_roll = block_roll.unwrap_or_default();
        let indexes = jj::get_int_array(obj, "reRolledDice")?;
        self.re_rolled_dice_indexes_null = indexes.is_none();
        self.re_rolled_dice_indexes = indexes.unwrap_or_default();
        self.team_re_roll_option = jj::get_boolean(obj, "teamReRollOption")?;
        self.pro_re_roll_option = jj::get_boolean(obj, "proReRollOption")?;
        self.brawler_option = jj::get_boolean(obj, "brawlerOption")?;
        if jj::is_defined_in(obj, "consummateOption") {
            self.consummate_option = jj::get_boolean(obj, "consummateOption")?;
        }
        self.single_use_re_roll_source =
            jj::get_enum_name(obj, "reRollSourceSingleUse")?.and_then(ReRollSourceFactory::for_name_shared).cloned();
        self.re_roll_explicit_die_skills = jj::get_json_array_option(obj, "skillArray")?
            .iter()
            .map(|v| SkillFactory::from_json_value_in(rules, Some(v)))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(())
    }

    /// Java `toJsonValue()`.
    pub fn to_json_value(&self, rules: crate::enums::Rules) -> Result<serde_json::Value, String> {
        use crate::factory::skill_factory::SkillFactory;
        use crate::util::java_json as jj;
        use serde_json::Value;
        let mut o = serde_json::Map::new();
        o.insert("dialogId".into(), Value::from(DialogId::BLOCK_ROLL_PARTIAL_RE_ROLL.get_name()));
        o.insert("choosingTeamId".into(), jj::string_or_null(self.choosing_team_id.as_deref()));
        o.insert("nrOfDice".into(), Value::from(self.nr_of_dice));
        o.insert("blockRoll".into(), jj::int_array_or_null((!self.block_roll_null).then_some(self.block_roll.as_slice())));
        o.insert(
            "reRolledDice".into(),
            jj::int_array_or_null((!self.re_rolled_dice_indexes_null).then_some(self.re_rolled_dice_indexes.as_slice())),
        );
        o.insert("teamReRollOption".into(), Value::Bool(self.team_re_roll_option));
        o.insert("proReRollOption".into(), Value::Bool(self.pro_re_roll_option));
        o.insert("brawlerOption".into(), Value::Bool(self.brawler_option));
        o.insert(
            "reRollSourceSingleUse".into(),
            jj::string_or_null(self.single_use_re_roll_source.as_ref().map(|s| s.name.as_str())),
        );
        o.insert("consummateOption".into(), Value::Bool(self.consummate_option));
        let skills = self
            .re_roll_explicit_die_skills
            .iter()
            .map(|s| SkillFactory::to_json_value_in(rules, *s))
            .collect::<Result<Vec<_>, _>>()?;
        o.insert("skillArray".into(), Value::Array(skills));
        Ok(Value::Object(o))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::Rules;
    use serde_json::json;

    fn round_trip(input: serde_json::Value) -> Result<serde_json::Value, String> {
        let mut p = DialogBlockRollPartialReRollParameter::default();
        p.init_from(Rules::Bb2020, &input)?;
        p.to_json_value(Rules::Bb2020)
    }

    #[test]
    fn json_round_trip_matches_java() {
        let input = json!({"dialogId": "blockRollPartialReRoll", "choosingTeamId": "t", "nrOfDice": 2,
            "blockRoll": [3, 6], "reRolledDice": [1], "teamReRollOption": true, "proReRollOption": false,
            "brawlerOption": false, "reRollSourceSingleUse": null, "consummateOption": false, "skillArray": []});
        assert_eq!(round_trip(input.clone()).unwrap(), input);
    }

    #[test]
    fn optional_members_and_unknown_skills() {
        // consummateOption optional, skillArray absent -> empty, unknown skill -> null element,
        // absent int arrays -> null, a known source canonicalised.
        let out = round_trip(json!({"dialogId": "blockRollPartialReRoll", "teamReRollOption": false,
            "proReRollOption": true, "brawlerOption": true, "reRollSourceSingleUse": "lord of chaos"})).unwrap();
        assert_eq!(out["consummateOption"], json!(false));
        assert_eq!(out["skillArray"], json!([]));
        assert_eq!(out["blockRoll"], json!(null));
        assert_eq!(out["reRolledDice"], json!(null));
        assert_eq!(out["reRollSourceSingleUse"], json!("Lord of Chaos"));
        let out = round_trip(json!({"dialogId": "blockRollPartialReRoll", "teamReRollOption": false,
            "proReRollOption": false, "brawlerOption": false, "skillArray": ["Brawler", "Nope", null]})).unwrap();
        assert_eq!(out["skillArray"], json!(["Brawler", null, null]));
        assert!(round_trip(json!({"dialogId": "blockRollPartialReRoll", "teamReRollOption": false, "proReRollOption": false})).is_err());
    }
}
