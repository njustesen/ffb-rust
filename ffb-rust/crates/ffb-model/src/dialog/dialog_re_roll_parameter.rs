use serde::{Deserialize, Serialize};
use crate::enums::ReRollSource;
use crate::enums::SkillId;
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogReRollParameter.
/// Note: ReRolledAction serialized as String; CommonProperty serialized as String key.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogReRollParameter {
    pub player_id: Option<String>,
    pub default_value_key: Option<String>,
    /// ReRolledAction serialized by name.
    pub re_rolled_action: Option<String>,
    pub minimum_roll: i32,
    pub team_re_roll_option: bool,
    pub pro_re_roll_option: bool,
    pub fumble: bool,
    pub single_use_re_roll_source: Option<ReRollSource>,
    pub re_roll_skill: Option<SkillId>,
    pub modifying_skill: Option<SkillId>,
    pub messages: Vec<String>,
    /// CommonProperty serialized by key.
    pub menu_property: Option<String>,
}

impl DialogReRollParameter {
    pub fn get_player_id(&self) -> Option<&str> { self.player_id.as_deref() }
    pub fn get_default_value_key(&self) -> Option<&str> { self.default_value_key.as_deref() }
    pub fn get_re_rolled_action(&self) -> Option<&str> { self.re_rolled_action.as_deref() }
    pub fn get_minimum_roll(&self) -> i32 { self.minimum_roll }
    pub fn is_team_re_roll_option(&self) -> bool { self.team_re_roll_option }
    pub fn is_pro_re_roll_option(&self) -> bool { self.pro_re_roll_option }
    pub fn is_fumble(&self) -> bool { self.fumble }
    pub fn get_messages(&self) -> &[String] { &self.messages }
    pub fn get_menu_property(&self) -> Option<&str> { self.menu_property.as_deref() }
}

impl IDialogParameter for DialogReRollParameter {
    fn get_id(&self) -> DialogId { DialogId::RE_ROLL }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_round_trip() {
        let p = DialogReRollParameter {
            player_id: Some("p42".into()),
            minimum_roll: 4,
            team_re_roll_option: true,
            re_roll_skill: Some(SkillId::Pro),
            messages: vec!["Roll for it!".into()],
            ..Default::default()
        };
        let json = serde_json::to_string(&p).unwrap();
        let back: DialogReRollParameter = serde_json::from_str(&json).unwrap();
        assert_eq!(back.get_player_id(), Some("p42"));
        assert_eq!(back.get_minimum_roll(), 4);
        assert!(back.is_team_re_roll_option());
        assert_eq!(back.re_roll_skill, Some(SkillId::Pro));
    }

    #[test]
    fn get_id_is_re_roll() {
        let p = DialogReRollParameter::default();
        assert_eq!(p.get_id(), DialogId::RE_ROLL);
    }

    #[test]
    fn transform_preserves_data() {
        let p = DialogReRollParameter { fumble: true, minimum_roll: 2, ..Default::default() };
        let t = p.transform();
        assert_eq!(t.get_id(), DialogId::RE_ROLL);
    }
}
