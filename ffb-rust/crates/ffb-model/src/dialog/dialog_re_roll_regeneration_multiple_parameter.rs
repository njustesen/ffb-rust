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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dialog_id_is_re_roll_regeneration_multiple() {
        assert_eq!(DialogReRollRegenerationMultipleParameter::default().get_id(), DialogId::RE_ROLL_REGENERATION_MULTIPLE);
    }
    #[test]
    fn stores_player_ids_vec() {
        let p = DialogReRollRegenerationMultipleParameter { player_ids: vec!["p1".into(), "p2".into()], ..Default::default() };
        assert_eq!(p.get_player_ids().len(), 2);
    }

    #[test]
    fn default_is_sensible() {
        let p = DialogReRollRegenerationMultipleParameter::default();
        assert!(p.get_player_ids().is_empty());
        assert!(p.get_inducement_type().is_none());
    }

    #[test]
    fn inducement_type_accessor() {
        let p = DialogReRollRegenerationMultipleParameter {
            inducement_type: Some("APOTHECARY".into()),
            ..Default::default()
        };
        assert_eq!(p.get_inducement_type(), Some("APOTHECARY"));
    }

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
