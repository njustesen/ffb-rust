use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::enums::ReRollProperty;
use super::dialog_id::DialogId;
use super::has_re_roll_properties::HasReRollProperties;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogBlockRollPropertiesParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogBlockRollPropertiesParameter {
    pub choosing_team_id: Option<String>,
    pub nr_of_dice: i32,
    pub block_roll: Vec<i32>,
    pub re_roll_properties: Vec<ReRollProperty>,
    /// Maps re-rolled action name to source name.
    pub rr_action_to_source: HashMap<String, String>,
}

impl DialogBlockRollPropertiesParameter {
    pub fn get_choosing_team_id(&self) -> Option<&str> { self.choosing_team_id.as_deref() }
    pub fn get_nr_of_dice(&self) -> i32 { self.nr_of_dice }
    pub fn get_block_roll(&self) -> &[i32] { &self.block_roll }
    pub fn get_re_roll_properties(&self) -> &[ReRollProperty] { &self.re_roll_properties }
    pub fn get_rr_action_to_source(&self) -> &HashMap<String, String> { &self.rr_action_to_source }

    /// Java: hasActualReRoll().
    pub fn has_actual_re_roll(&self) -> bool {
        self.re_roll_properties.iter().any(|p| p.is_actual_reroll()) || !self.rr_action_to_source.is_empty()
    }
}

impl HasReRollProperties for DialogBlockRollPropertiesParameter {
    fn has_property(&self, property: ReRollProperty) -> bool {
        self.re_roll_properties.contains(&property)
    }
}

impl IDialogParameter for DialogBlockRollPropertiesParameter {
    fn get_id(&self) -> DialogId { DialogId::BLOCK_ROLL_PROPERTIES }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn has_property_true_when_present() {
        let p = DialogBlockRollPropertiesParameter {
            re_roll_properties: vec![ReRollProperty::Trr],
            ..Default::default()
        };
        assert!(p.has_property(ReRollProperty::Trr));
        assert!(!p.has_property(ReRollProperty::Loner));
    }

    #[test]
    fn has_actual_re_roll_true_for_trr() {
        let p = DialogBlockRollPropertiesParameter {
            re_roll_properties: vec![ReRollProperty::Trr],
            ..Default::default()
        };
        assert!(p.has_actual_re_roll());
    }

    #[test]
    fn has_actual_re_roll_false_for_loner_only() {
        let p = DialogBlockRollPropertiesParameter {
            re_roll_properties: vec![ReRollProperty::Loner],
            ..Default::default()
        };
        assert!(!p.has_actual_re_roll());
    }

    #[test]
    fn has_actual_re_roll_true_when_rr_action_to_source_nonempty() {
        let p = DialogBlockRollPropertiesParameter {
            rr_action_to_source: [("block".to_string(), "team".to_string())].into(),
            ..Default::default()
        };
        assert!(p.has_actual_re_roll());
    }

    #[test]
    fn has_actual_re_roll_false_when_empty() {
        let p = DialogBlockRollPropertiesParameter::default();
        assert!(!p.has_actual_re_roll());
    }
}
