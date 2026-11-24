use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogKickOffResultParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogKickOffResultParameter {
    pub team_id: Option<String>,
}

impl DialogKickOffResultParameter {
    pub fn get_team_id(&self) -> Option<&str> { self.team_id.as_deref() }
}

impl IDialogParameter for DialogKickOffResultParameter {
    fn get_id(&self) -> DialogId { DialogId::KICK_OFF_RESULT }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}

// JSON serialization (BACKLOG §O).
impl DialogKickOffResultParameter {
    /// Java `initFrom(IFactorySource, JsonValue)`.
    pub fn init_from(&mut self, _rules: crate::enums::Rules, json: &serde_json::Value) -> Result<(), String> {
        use crate::util::java_json as jj;
        let obj = jj::to_json_object(json)?;
        super::util_dialog_parameter::validate_dialog_id_json(DialogId::KICK_OFF_RESULT, obj)?;
        self.team_id = jj::get_string(obj, "teamId")?;
        Ok(())
    }

    /// Java `toJsonValue()`: dialogId, teamId.
    pub fn to_json_value(&self, _rules: crate::enums::Rules) -> Result<serde_json::Value, String> {
        use crate::util::java_json as jj;
        let mut o = serde_json::Map::new();
        o.insert("dialogId".into(), serde_json::Value::from(DialogId::KICK_OFF_RESULT.get_name()));
        o.insert("teamId".into(), jj::string_or_null(self.team_id.as_deref()));
        Ok(serde_json::Value::Object(o))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn serde_round_trip() {
        let p = DialogKickOffResultParameter { team_id: Some("home".into()) };
        let json = serde_json::to_string(&p).unwrap();
        let back: DialogKickOffResultParameter = serde_json::from_str(&json).unwrap();
        assert_eq!(back.get_team_id(), Some("home"));
    }

}
