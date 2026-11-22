use serde::{Deserialize, Serialize};
use crate::enums::{PlayerState, SeriousInjuryKind, ApothecaryType};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogUseApothecaryParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogUseApothecaryParameter {
    pub player_id: Option<String>,
    pub player_state: Option<PlayerState>,
    pub serious_injury: Option<SeriousInjuryKind>,
    pub apothecary_types: Vec<ApothecaryType>,
}

impl DialogUseApothecaryParameter {
    pub fn get_player_id(&self) -> Option<&str> { self.player_id.as_deref() }
    pub fn get_player_state(&self) -> Option<PlayerState> { self.player_state }
    pub fn get_serious_injury(&self) -> Option<SeriousInjuryKind> { self.serious_injury }
    pub fn get_apothecary_types(&self) -> &[ApothecaryType] { &self.apothecary_types }
}

impl IDialogParameter for DialogUseApothecaryParameter {
    fn get_id(&self) -> DialogId { DialogId::USE_APOTHECARY }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}

// JSON serialization (BACKLOG §O).
impl DialogUseApothecaryParameter {
    /// Java `initFrom(IFactorySource, JsonValue)`.
    pub fn init_from(&mut self, rules: crate::enums::Rules, json: &serde_json::Value) -> Result<(), String> {
        use crate::factory::serious_injury_factory as sif;
        use crate::util::java_json as jj;
        let obj = jj::to_json_object(json)?;
        super::util_dialog_parameter::validate_dialog_id_json(DialogId::USE_APOTHECARY, obj)?;
        self.player_id = jj::get_string(obj, "playerId")?;
        self.player_state = jj::get_player_state(obj, "playerState")?;
        self.serious_injury = sif::kind_from_json(rules, obj, "seriousInjury")?;
        if jj::is_defined_in(obj, "apothecaryTypes") {
            for name in crate::bb2020::injury_description::read_apothecary_types(obj)? {
                self.apothecary_types.push(ApothecaryType::from_name(&name).ok_or("unreachable apothecary type")?);
            }
        }
        Ok(())
    }

    /// Java `toJsonValue()`.
    pub fn to_json_value(&self, rules: crate::enums::Rules) -> Result<serde_json::Value, String> {
        use crate::factory::serious_injury_factory as sif;
        use crate::util::java_json as jj;
        use serde_json::Value;
        let mut o = serde_json::Map::new();
        o.insert("dialogId".into(), Value::from(DialogId::USE_APOTHECARY.get_name()));
        o.insert("playerId".into(), jj::string_or_null(self.player_id.as_deref()));
        o.insert("playerState".into(), jj::player_state_or_null(self.player_state.map(|s| s.0 as i32)));
        o.insert("seriousInjury".into(), sif::kind_to_json(rules, self.serious_injury)?);
        o.insert("apothecaryTypes".into(), Value::Array(self.apothecary_types.iter().map(|t| Value::from(t.name())).collect()));
        Ok(Value::Object(o))
    }
}
