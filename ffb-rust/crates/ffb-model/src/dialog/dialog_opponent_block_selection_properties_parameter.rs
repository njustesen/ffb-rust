use serde::{Deserialize, Serialize};
use crate::model::block_roll_properties::BlockRollProperties;
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogOpponentBlockSelectionPropertiesParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogOpponentBlockSelectionPropertiesParameter {
    pub team_id: Option<String>,
    /// Java `List<BlockRollProperties> blockRolls`.
    pub block_rolls: Vec<BlockRollProperties>,
}

impl DialogOpponentBlockSelectionPropertiesParameter {
    pub fn get_team_id(&self) -> Option<&str> { self.team_id.as_deref() }
    pub fn get_block_rolls(&self) -> &[BlockRollProperties] { &self.block_rolls }
}

impl IDialogParameter for DialogOpponentBlockSelectionPropertiesParameter {
    fn get_id(&self) -> DialogId { DialogId::OPPONENT_BLOCK_SELECTION_PROPERTIES }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}

// JSON serialization (BACKLOG §O).
impl DialogOpponentBlockSelectionPropertiesParameter {
    /// Java `initFrom(IFactorySource, JsonValue)`: every `blockRolls` element a new
    /// BlockRollProperties; the team id travels under `playerId`.
    pub fn init_from(&mut self, rules: crate::enums::Rules, json: &serde_json::Value) -> Result<(), String> {
        use crate::util::java_json as jj;
        let obj = jj::to_json_object(json)?;
        super::util_dialog_parameter::validate_dialog_id_json(DialogId::OPPONENT_BLOCK_SELECTION_PROPERTIES, obj)?;
        self.block_rolls = jj::get_json_array_option(obj, "blockRolls")?
            .iter()
            .map(|roll| {
                let mut p = BlockRollProperties::new();
                p.init_from(rules, roll).map(|_| p)
            })
            .collect::<Result<Vec<_>, _>>()?;
        self.team_id = jj::get_string(obj, "playerId")?;
        Ok(())
    }

    /// Java `toJsonValue()`: dialogId, blockRolls, playerId (= teamId).
    pub fn to_json_value(&self, _rules: crate::enums::Rules) -> Result<serde_json::Value, String> {
        use crate::util::java_json as jj;
        use serde_json::Value;
        let mut o = serde_json::Map::new();
        o.insert("dialogId".into(), Value::from(DialogId::OPPONENT_BLOCK_SELECTION_PROPERTIES.get_name()));
        let rolls = self.block_rolls.iter().map(BlockRollProperties::to_json_value).collect::<Result<Vec<_>, _>>()?;
        o.insert("blockRolls".into(), Value::Array(rolls));
        o.insert("playerId".into(), jj::string_or_null(self.team_id.as_deref()));
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
        let roll = json!({"blockRoll":[5,4],"blockRollId":0,"doubleTargetStrength":false,"isOwnChoice":false,"nrOfDice":2,
            "playerId":"p2","playerStateOld":257,"proIndex":0,"reRollActionToSourceMap":{},"reRollProperties":[],
            "reRolledDice":[],"selectedIndex":-1,"successfulDauntless":false});
        let input = json!({"dialogId":"opponentBlockSelectionProperties","playerId":"teamAway","blockRolls":[roll]});
        let p = DialogParameterFactory::new().for_json_value(Rules::Bb2025, &input).unwrap().unwrap();
        assert_eq!(p.to_json_value(Rules::Bb2025).unwrap(), input);
    }
}
