use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogPileDriverParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogPileDriverParameter {
    pub knocked_down_players: Vec<String>,
    pub team_id: Option<String>,
}

impl DialogPileDriverParameter {
    pub fn get_knocked_down_players(&self) -> &[String] { &self.knocked_down_players }
    pub fn get_team_id(&self) -> Option<&str> { self.team_id.as_deref() }
    pub fn add_knocked_down_player(&mut self, id: impl Into<String>) {
        let s = id.into();
        if !s.is_empty() { self.knocked_down_players.push(s); }
    }
}

impl IDialogParameter for DialogPileDriverParameter {
    fn get_id(&self) -> DialogId { DialogId::PILE_DRIVER }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}

// JSON serialization (BACKLOG §O).
impl DialogPileDriverParameter {
    /// Java `initFrom(IFactorySource, JsonValue)`.
    pub fn init_from(&mut self, _rules: crate::enums::Rules, json: &serde_json::Value) -> Result<(), String> {
        use crate::util::java_json as jj;
        let obj = jj::to_json_object(json)?;
        super::util_dialog_parameter::validate_dialog_id_json(DialogId::PILE_DRIVER, obj)?;
        self.team_id = jj::get_string(obj, "teamId")?;
        // Arrays.stream(PLAYER_IDS.getFrom(..)): absent/null -> NPE. A JSON-null element would be
        // a null list entry, which this Vec<String> cannot hold (none occur) -> Err.
        let ids = jj::get_string_array(obj, "playerIds")?.ok_or("playerIds: NullPointerException (Arrays.stream(null))")?;
        for id in ids {
            self.knocked_down_players.push(id.ok_or("playerIds: null element not representable")?);
        }
        Ok(())
    }

    /// Java `toJsonValue()`.
    pub fn to_json_value(&self, _rules: crate::enums::Rules) -> Result<serde_json::Value, String> {
        use crate::util::java_json as jj;
        use serde_json::Value;
        let mut o = serde_json::Map::new();
        o.insert("dialogId".into(), Value::from(DialogId::PILE_DRIVER.get_name()));
        o.insert("playerIds".into(), jj::string_array(&self.knocked_down_players));
        o.insert("teamId".into(), jj::string_or_null(self.team_id.as_deref()));
        Ok(Value::Object(o))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_knocked_down_player_appends_nonempty() {
        let mut p = DialogPileDriverParameter::default();
        p.add_knocked_down_player("p1");
        p.add_knocked_down_player("");
        assert_eq!(p.get_knocked_down_players(), &["p1"]);
    }

    #[test]
    fn add_empty_string_is_ignored() {
        let mut p = DialogPileDriverParameter::default();
        p.add_knocked_down_player("");
        assert!(p.get_knocked_down_players().is_empty());
    }
}
