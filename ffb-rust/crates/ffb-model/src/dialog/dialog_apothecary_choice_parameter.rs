use serde::{Deserialize, Serialize};
use crate::enums::PlayerState;
use crate::enums::SeriousInjuryKind;
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogApothecaryChoiceParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogApothecaryChoiceParameter {
    pub player_id: Option<String>,
    pub player_state_old: Option<PlayerState>,
    pub serious_injury_old: Option<SeriousInjuryKind>,
    pub player_state_new: Option<PlayerState>,
    pub serious_injury_new: Option<SeriousInjuryKind>,
}

impl DialogApothecaryChoiceParameter {
    pub fn new() -> Self { Self::default() }

    pub fn new_with(
        player_id: impl Into<String>,
        player_state_old: Option<PlayerState>,
        serious_injury_old: Option<SeriousInjuryKind>,
        player_state_new: Option<PlayerState>,
        serious_injury_new: Option<SeriousInjuryKind>,
    ) -> Self {
        DialogApothecaryChoiceParameter {
            player_id: Some(player_id.into()),
            player_state_old,
            serious_injury_old,
            player_state_new,
            serious_injury_new,
        }
    }

    pub fn get_player_id(&self) -> Option<&str> { self.player_id.as_deref() }
    pub fn get_player_state_old(&self) -> Option<PlayerState> { self.player_state_old }
    pub fn get_serious_injury_old(&self) -> Option<SeriousInjuryKind> { self.serious_injury_old }
    pub fn get_player_state_new(&self) -> Option<PlayerState> { self.player_state_new }
    pub fn get_serious_injury_new(&self) -> Option<SeriousInjuryKind> { self.serious_injury_new }
}

impl IDialogParameter for DialogApothecaryChoiceParameter {
    fn get_id(&self) -> DialogId { DialogId::APOTHECARY_CHOICE }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}

// JSON serialization (BACKLOG §O).
impl DialogApothecaryChoiceParameter {
    /// Java `initFrom(IFactorySource, JsonValue)`.
    pub fn init_from(&mut self, rules: crate::enums::Rules, json: &serde_json::Value) -> Result<(), String> {
        use crate::factory::serious_injury_factory as sif;
        use crate::util::java_json as jj;
        let obj = jj::to_json_object(json)?;
        super::util_dialog_parameter::validate_dialog_id_json(DialogId::APOTHECARY_CHOICE, obj)?;
        self.player_id = jj::get_string(obj, "playerId")?;
        self.player_state_old = jj::get_player_state(obj, "playerStateOld")?;
        self.serious_injury_old = sif::kind_from_json(rules, obj, "seriousInjuryOld")?;
        self.player_state_new = jj::get_player_state(obj, "playerStateNew")?;
        self.serious_injury_new = sif::kind_from_json(rules, obj, "seriousInjuryNew")?;
        Ok(())
    }

    /// Java `toJsonValue()`.
    pub fn to_json_value(&self, rules: crate::enums::Rules) -> Result<serde_json::Value, String> {
        use crate::factory::serious_injury_factory as sif;
        use crate::util::java_json as jj;
        use serde_json::Value;
        let mut o = serde_json::Map::new();
        o.insert("dialogId".into(), Value::from(DialogId::APOTHECARY_CHOICE.get_name()));
        o.insert("playerId".into(), jj::string_or_null(self.player_id.as_deref()));
        o.insert("playerStateOld".into(), jj::player_state_or_null(self.player_state_old.map(|s| s.0 as i32)));
        o.insert("seriousInjuryOld".into(), sif::kind_to_json(rules, self.serious_injury_old)?);
        o.insert("playerStateNew".into(), jj::player_state_or_null(self.player_state_new.map(|s| s.0 as i32)));
        o.insert("seriousInjuryNew".into(), sif::kind_to_json(rules, self.serious_injury_new)?);
        Ok(Value::Object(o))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_round_trip() {
        let p = DialogApothecaryChoiceParameter::new_with("p42", None, None, None, None);
        let json = serde_json::to_string(&p).unwrap();
        let back: DialogApothecaryChoiceParameter = serde_json::from_str(&json).unwrap();
        assert_eq!(back.get_player_id(), Some("p42"));
    }
}
