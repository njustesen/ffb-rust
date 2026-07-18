use serde::{Deserialize, Serialize};
use crate::enums::ReRollProperty;
use crate::enums::SkillId;
use super::dialog_id::DialogId;
use super::has_re_roll_properties::HasReRollProperties;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogReRollPropertiesParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogReRollPropertiesParameter {
    pub player_id: Option<String>,
    pub default_value_key: Option<String>,
    /// ReRolledAction serialized by name.
    pub re_rolled_action: Option<String>,
    pub minimum_roll: i32,
    pub fumble: bool,
    pub re_roll_skill: Option<SkillId>,
    pub modifying_skill: Option<SkillId>,
    pub messages: Vec<String>,
    pub re_roll_properties: Vec<ReRollProperty>,
    /// CommonProperty serialized by key.
    pub menu_property: Option<String>,
}

impl DialogReRollPropertiesParameter {
    pub fn get_player_id(&self) -> Option<&str> { self.player_id.as_deref() }
    pub fn get_default_value_key(&self) -> Option<&str> { self.default_value_key.as_deref() }
    pub fn get_re_rolled_action(&self) -> Option<&str> { self.re_rolled_action.as_deref() }
    pub fn get_minimum_roll(&self) -> i32 { self.minimum_roll }
    pub fn is_fumble(&self) -> bool { self.fumble }
    pub fn get_messages(&self) -> &[String] { &self.messages }
    pub fn get_re_roll_properties(&self) -> &[ReRollProperty] { &self.re_roll_properties }
    pub fn get_menu_property(&self) -> Option<&str> { self.menu_property.as_deref() }
}

impl HasReRollProperties for DialogReRollPropertiesParameter {
    fn has_property(&self, property: ReRollProperty) -> bool {
        self.re_roll_properties.contains(&property)
    }
}

impl IDialogParameter for DialogReRollPropertiesParameter {
    fn get_id(&self) -> DialogId { DialogId::RE_ROLL_PROPERTIES }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dialog_id_is_re_roll_properties() {
        assert_eq!(DialogReRollPropertiesParameter::default().get_id(), DialogId::RE_ROLL_PROPERTIES);
    }
    #[test]
    fn stores_minimum_roll_and_fumble() {
        let p = DialogReRollPropertiesParameter { minimum_roll: 3, fumble: true, ..Default::default() };
        assert_eq!(p.get_minimum_roll(), 3);
        assert!(p.is_fumble());
    }

    #[test]
    fn default_is_sensible() {
        let p = DialogReRollPropertiesParameter::default();
        assert!(p.get_player_id().is_none());
        assert!(p.get_default_value_key().is_none());
        assert!(p.get_re_rolled_action().is_none());
        assert_eq!(p.get_minimum_roll(), 0);
        assert!(!p.is_fumble());
        assert!(p.get_messages().is_empty());
        assert!(p.get_re_roll_properties().is_empty());
        assert!(p.get_menu_property().is_none());
    }

    #[test]
    fn messages_and_player_id_stored() {
        let p = DialogReRollPropertiesParameter {
            player_id: Some("p1".into()),
            messages: vec!["msg1".into(), "msg2".into()],
            ..Default::default()
        };
        assert_eq!(p.get_player_id(), Some("p1"));
        assert_eq!(p.get_messages().len(), 2);
        assert_eq!(p.get_messages()[0], "msg1");
    }

    #[test]
    fn menu_property_and_default_value_key() {
        let p = DialogReRollPropertiesParameter {
            menu_property: Some("prop_key".into()),
            default_value_key: Some("def_key".into()),
            ..Default::default()
        };
        assert_eq!(p.get_menu_property(), Some("prop_key"));
        assert_eq!(p.get_default_value_key(), Some("def_key"));
    }

    #[test]
    fn has_property_true_when_present() {
        let p = DialogReRollPropertiesParameter {
            re_roll_properties: vec![ReRollProperty::Trr],
            ..Default::default()
        };
        assert!(p.has_property(ReRollProperty::Trr));
        assert!(!p.has_property(ReRollProperty::Loner));
    }

    #[test]
    fn has_property_false_when_absent() {
        let p = DialogReRollPropertiesParameter::default();
        assert!(!p.has_property(ReRollProperty::Trr));
    }
}
