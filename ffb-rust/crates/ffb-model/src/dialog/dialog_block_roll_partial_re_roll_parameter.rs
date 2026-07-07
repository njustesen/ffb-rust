use serde::{Deserialize, Serialize};
use crate::enums::ReRollSource;
use crate::enums::SkillId;
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogBlockRollPartialReRollParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogBlockRollPartialReRollParameter {
    pub choosing_team_id: Option<String>,
    pub nr_of_dice: i32,
    pub block_roll: Vec<i32>,
    pub re_rolled_dice_indexes: Vec<i32>,
    pub team_re_roll_option: bool,
    pub pro_re_roll_option: bool,
    pub brawler_option: bool,
    pub consummate_option: bool,
    pub single_use_re_roll_source: Option<ReRollSource>,
    pub re_roll_explicit_die_skills: Vec<SkillId>,
}

impl DialogBlockRollPartialReRollParameter {
    pub fn get_choosing_team_id(&self) -> Option<&str> { self.choosing_team_id.as_deref() }
    pub fn get_nr_of_dice(&self) -> i32 { self.nr_of_dice }
    pub fn get_block_roll(&self) -> &[i32] { &self.block_roll }
    pub fn get_re_rolled_dice_indexes(&self) -> &[i32] { &self.re_rolled_dice_indexes }
    pub fn has_team_re_roll_option(&self) -> bool { self.team_re_roll_option }
    pub fn has_pro_re_roll_option(&self) -> bool { self.pro_re_roll_option }
    pub fn has_brawler_option(&self) -> bool { self.brawler_option }
    pub fn has_consummate_option(&self) -> bool { self.consummate_option }
}

impl IDialogParameter for DialogBlockRollPartialReRollParameter {
    fn get_id(&self) -> DialogId { DialogId::BLOCK_ROLL_PARTIAL_RE_ROLL }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dialog_id_is_block_roll_partial_re_roll() {
        assert_eq!(DialogBlockRollPartialReRollParameter::default().get_id(), DialogId::BLOCK_ROLL_PARTIAL_RE_ROLL);
    }

    #[test]
    fn boolean_flags_propagate_correctly() {
        let p = DialogBlockRollPartialReRollParameter {
            team_re_roll_option: true,
            pro_re_roll_option: false,
            brawler_option: true,
            consummate_option: false,
            ..Default::default()
        };
        assert!(p.has_team_re_roll_option());
        assert!(!p.has_pro_re_roll_option());
        assert!(p.has_brawler_option());
        assert!(!p.has_consummate_option());
    }

    #[test]
    fn stores_dice_and_team_id() {
        let p = DialogBlockRollPartialReRollParameter {
            choosing_team_id: Some("team1".into()),
            nr_of_dice: 2,
            block_roll: vec![3, 5],
            ..Default::default()
        };
        assert_eq!(p.get_choosing_team_id(), Some("team1"));
        assert_eq!(p.get_nr_of_dice(), 2);
        assert_eq!(p.get_block_roll(), &[3, 5]);
    }

    #[test]
    fn transform_preserves_id() {
        let t = DialogBlockRollPartialReRollParameter::default().transform();
        assert_eq!(t.get_id(), DialogId::BLOCK_ROLL_PARTIAL_RE_ROLL);
    }

    #[test]
    fn re_rolled_dice_indexes_stored() {
        let p = DialogBlockRollPartialReRollParameter {
            re_rolled_dice_indexes: vec![0, 2],
            ..Default::default()
        };
        assert_eq!(p.get_re_rolled_dice_indexes(), &[0, 2]);
    }
}
