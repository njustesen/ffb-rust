use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogBlockRollParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogBlockRollParameter {
    pub choosing_team_id: Option<String>,
    pub nr_of_dice: i32,
    pub block_roll: Vec<i32>,
    /// Java: `fBlockRoll` is null (JSON `blockRoll` absent or null); `block_roll` is then empty.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub block_roll_null: bool,
    pub team_re_roll_option: bool,
    pub pro_re_roll_option: bool,
}

impl DialogBlockRollParameter {
    pub fn get_choosing_team_id(&self) -> Option<&str> { self.choosing_team_id.as_deref() }
    pub fn get_nr_of_dice(&self) -> i32 { self.nr_of_dice }
    pub fn get_block_roll(&self) -> &[i32] { &self.block_roll }
    pub fn has_team_re_roll_option(&self) -> bool { self.team_re_roll_option }
    pub fn has_pro_re_roll_option(&self) -> bool { self.pro_re_roll_option }
}

impl IDialogParameter for DialogBlockRollParameter {
    fn get_id(&self) -> DialogId { DialogId::BLOCK_ROLL }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}

// JSON serialization (BACKLOG §O).
impl DialogBlockRollParameter {
    /// Java `initFrom(IFactorySource, JsonValue)`.
    pub fn init_from(&mut self, _rules: crate::enums::Rules, json: &serde_json::Value) -> Result<(), String> {
        use crate::util::java_json as jj;
        let obj = jj::to_json_object(json)?;
        super::util_dialog_parameter::validate_dialog_id_json(DialogId::BLOCK_ROLL, obj)?;
        self.choosing_team_id = jj::get_string(obj, "choosingTeamId")?;
        self.nr_of_dice = jj::get_int(obj, "nrOfDice")?;
        let block_roll = jj::get_int_array(obj, "blockRoll")?;
        self.block_roll_null = block_roll.is_none();
        self.block_roll = block_roll.unwrap_or_default();
        self.team_re_roll_option = jj::get_boolean(obj, "teamReRollOption")?;
        self.pro_re_roll_option = jj::get_boolean(obj, "proReRollOption")?;
        Ok(())
    }

    /// Java `toJsonValue()`.
    pub fn to_json_value(&self, _rules: crate::enums::Rules) -> Result<serde_json::Value, String> {
        use crate::util::java_json as jj;
        use serde_json::Value;
        let mut o = serde_json::Map::new();
        o.insert("dialogId".into(), Value::from(DialogId::BLOCK_ROLL.get_name()));
        o.insert("choosingTeamId".into(), jj::string_or_null(self.choosing_team_id.as_deref()));
        o.insert("nrOfDice".into(), Value::from(self.nr_of_dice));
        o.insert("blockRoll".into(), jj::int_array_or_null((!self.block_roll_null).then_some(self.block_roll.as_slice())));
        o.insert("teamReRollOption".into(), Value::Bool(self.team_re_roll_option));
        o.insert("proReRollOption".into(), Value::Bool(self.pro_re_roll_option));
        Ok(Value::Object(o))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_round_trip() {
        let p = DialogBlockRollParameter {
            choosing_team_id: Some("teamA".into()),
            nr_of_dice: 2,
            block_roll: vec![3, 5],
            team_re_roll_option: true,
            pro_re_roll_option: false,
            ..Default::default()
        };
        let json = serde_json::to_string(&p).unwrap();
        let back: DialogBlockRollParameter = serde_json::from_str(&json).unwrap();
        assert_eq!(back.get_choosing_team_id(), Some("teamA"));
        assert_eq!(back.get_nr_of_dice(), 2);
        assert_eq!(back.get_block_roll(), &[3, 5]);
        assert!(back.has_team_re_roll_option());
        assert!(!back.has_pro_re_roll_option());
    }


    fn round_trip(input: serde_json::Value) -> Result<serde_json::Value, String> {
        let mut p = DialogBlockRollParameter::default();
        p.init_from(crate::enums::Rules::Bb2016, &input)?;
        p.to_json_value(crate::enums::Rules::Bb2016)
    }

    #[test]
    fn json_round_trip_matches_java() {
        let input = serde_json::json!({"dialogId": "blockRoll", "choosingTeamId": "t1", "nrOfDice": -2,
            "blockRoll": [1, 6], "teamReRollOption": true, "proReRollOption": false});
        assert_eq!(round_trip(input.clone()).unwrap(), input);
        // absent blockRoll -> null int[] -> JSON null; missing primitive boolean throws.
        let out = round_trip(serde_json::json!({"dialogId": "blockRoll", "teamReRollOption": false, "proReRollOption": false})).unwrap();
        assert_eq!(out["blockRoll"], serde_json::Value::Null);
        assert_eq!(out["choosingTeamId"], serde_json::Value::Null);
        assert!(round_trip(serde_json::json!({"dialogId": "blockRoll", "teamReRollOption": false})).is_err());
        assert!(round_trip(serde_json::json!({"dialogId": "blockRoll", "blockRoll": ["x"], "teamReRollOption": false, "proReRollOption": false})).is_err());
    }
}
