use serde::{Deserialize, Serialize};
use crate::model::block_roll_properties::BlockRollProperties;
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogReRollBlockForTargetsPropertiesParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogReRollBlockForTargetsPropertiesParameter {
    pub player_id: Option<String>,
    /// Java `List<BlockRollProperties> blockRolls` (null for a default-constructed parameter;
    /// `initFrom` always sets a list).
    pub block_rolls: Vec<BlockRollProperties>,
}

impl DialogReRollBlockForTargetsPropertiesParameter {
    pub fn get_player_id(&self) -> Option<&str> { self.player_id.as_deref() }
    pub fn get_block_rolls(&self) -> &[BlockRollProperties] { &self.block_rolls }
}

impl IDialogParameter for DialogReRollBlockForTargetsPropertiesParameter {
    fn get_id(&self) -> DialogId { DialogId::RE_ROLL_BLOCK_FOR_TARGETS_PROPERTIES }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}

// JSON serialization (BACKLOG §O).
impl DialogReRollBlockForTargetsPropertiesParameter {
    /// Java `initFrom(IFactorySource, JsonValue)`.
    pub fn init_from(&mut self, rules: crate::enums::Rules, json: &serde_json::Value) -> Result<(), String> {
        use crate::util::java_json as jj;
        let obj = jj::to_json_object(json)?;
        super::util_dialog_parameter::validate_dialog_id_json(DialogId::RE_ROLL_BLOCK_FOR_TARGETS_PROPERTIES, obj)?;
        // JsonArrayOption: absent/null -> a new empty array; every element a new BlockRollProperties.
        self.block_rolls = jj::get_json_array_option(obj, "blockRolls")?
            .iter()
            .map(|roll| {
                let mut p = BlockRollProperties::new();
                p.init_from(rules, roll).map(|_| p)
            })
            .collect::<Result<Vec<_>, _>>()?;
        self.player_id = jj::get_string(obj, "playerId")?;
        Ok(())
    }

    /// Java `toJsonValue()`.
    pub fn to_json_value(&self, _rules: crate::enums::Rules) -> Result<serde_json::Value, String> {
        use crate::util::java_json as jj;
        use serde_json::Value;
        let mut o = serde_json::Map::new();
        o.insert("dialogId".into(), Value::from(DialogId::RE_ROLL_BLOCK_FOR_TARGETS_PROPERTIES.get_name()));
        let rolls = self.block_rolls.iter().map(BlockRollProperties::to_json_value).collect::<Result<Vec<_>, _>>()?;
        o.insert("blockRolls".into(), Value::Array(rolls));
        o.insert("playerId".into(), jj::string_or_null(self.player_id.as_deref()));
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
        let roll = json!({"blockRoll":[5,4],"blockRollId":0,"doubleTargetStrength":false,"isOwnChoice":true,"nrOfDice":2,
            "playerId":"p2","playerStateOld":257,"proIndex":0,"reRollActionToSourceMap":{},"reRollProperties":[],
            "reRolledDice":[],"selectedIndex":0,"successfulDauntless":false});
        let input = json!({"dialogId":"reRollBlockForTargetsProperties","playerId":"p1","blockRolls":[roll]});
        let p = DialogParameterFactory::new().for_json_value(Rules::Bb2025, &input).unwrap().unwrap();
        assert_eq!(p.to_json_value(Rules::Bb2025).unwrap(), input);
        // No blockRolls -> an empty list (JsonArrayOption), not null.
        let p = DialogParameterFactory::new()
            .for_json_value(Rules::Bb2025, &json!({"dialogId":"reRollBlockForTargetsProperties"}))
            .unwrap()
            .unwrap();
        assert_eq!(p.to_json_value(Rules::Bb2025).unwrap(), json!({"dialogId":"reRollBlockForTargetsProperties","blockRolls":[],"playerId":null}));
    }
}
