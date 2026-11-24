use serde::{Deserialize, Serialize};
use crate::enums::ReRollOptions;
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogReRollRegenerationMultipleParameter.
/// Note: InducementType serialized as String name (stub not yet translated).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogReRollRegenerationMultipleParameter {
    pub player_ids: Vec<String>,
    /// InducementType serialized by name.
    pub inducement_type: Option<String>,
    pub re_roll_options: Vec<ReRollOptions>,
}

impl DialogReRollRegenerationMultipleParameter {
    pub fn get_player_ids(&self) -> &[String] { &self.player_ids }
    pub fn get_inducement_type(&self) -> Option<&str> { self.inducement_type.as_deref() }
    pub fn get_re_roll_options(&self) -> &[ReRollOptions] { &self.re_roll_options }
}

impl IDialogParameter for DialogReRollRegenerationMultipleParameter {
    fn get_id(&self) -> DialogId { DialogId::RE_ROLL_REGENERATION_MULTIPLE }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}

// JSON serialization (BACKLOG §O).
impl DialogReRollRegenerationMultipleParameter {
    /// Java `initFrom(IFactorySource, JsonValue)`.
    pub fn init_from(&mut self, rules: crate::enums::Rules, json: &serde_json::Value) -> Result<(), String> {
        use crate::util::java_json as jj;
        let obj = jj::to_json_object(json)?;
        super::util_dialog_parameter::validate_dialog_id_json(DialogId::RE_ROLL_REGENERATION_MULTIPLE, obj)?;
        // `Arrays.asList(playerIds)` with no null check: absent -> NullPointerException.
        let ids = jj::get_string_array(obj, "playerIds")?
            .ok_or_else(|| "playerIds: null (NullPointerException)".to_string())?;
        self.player_ids = ids.into_iter().map(|id| id.unwrap_or_default()).collect();
        // JsonEnumWithNameOption over the ruleset's InducementTypeFactory; null or unknown -> null.
        self.inducement_type = jj::get_string(obj, "inducementType")?
            .and_then(|n| crate::factory::inducement_type_factory::for_name_in(rules, &n))
            .map(|t| t.id);
        Ok(())
    }

    /// Java `toJsonValue()`: dialogId, playerIds, inducementType (Java has no reRollOptions on the wire).
    pub fn to_json_value(&self, _rules: crate::enums::Rules) -> Result<serde_json::Value, String> {
        use crate::util::java_json as jj;
        use serde_json::Value;
        let mut o = serde_json::Map::new();
        o.insert("dialogId".to_string(), Value::String(DialogId::RE_ROLL_REGENERATION_MULTIPLE.get_name().to_string()));
        o.insert("playerIds".to_string(), jj::string_array(&self.player_ids));
        o.insert("inducementType".to_string(), jj::string_or_null(self.inducement_type.as_deref()));
        Ok(Value::Object(o))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn empty_player_ids_edge_case() {
        let p = DialogReRollRegenerationMultipleParameter { player_ids: vec![], inducement_type: None, re_roll_options: vec![] };
        assert!(p.get_player_ids().is_empty());
        assert_eq!(p.get_inducement_type(), None);
    }

    /// Java's DialogReRollRegenerationMultipleParameter carries a `reRollOptions` field
    /// (constructor, getter, transform pass-through, JSON round-trip). The Rust struct had
    /// dropped this field entirely — this test fails on that pre-fix behavior (field absent)
    /// and passes once the field is restored.
    #[test]
    fn re_roll_options_field_is_preserved() {
        use crate::enums::{ReRollOptions, ReRollProperty};
        let opts = ReRollOptions::new(vec![ReRollProperty::Trr]);
        let p = DialogReRollRegenerationMultipleParameter {
            player_ids: vec!["p1".into()],
            inducement_type: None,
            re_roll_options: vec![opts],
        };
        assert_eq!(p.get_re_roll_options().len(), 1);
        assert!(p.get_re_roll_options()[0].can_actually_reroll());

        // Serde round-trip must preserve the field too.
        let json = serde_json::to_string(&p).unwrap();
        let back: DialogReRollRegenerationMultipleParameter = serde_json::from_str(&json).unwrap();
        assert_eq!(back.get_re_roll_options().len(), 1);
    }
}
