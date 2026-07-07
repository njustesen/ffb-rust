use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogBlockRollParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogBlockRollParameter {
    pub choosing_team_id: Option<String>,
    pub nr_of_dice: i32,
    pub block_roll: Vec<i32>,
    pub team_re_roll_option: bool,
    pub pro_re_roll_option: bool,
}

impl DialogBlockRollParameter {
    pub fn get_choosing_team_id(&self) -> Option<&str> { self.choosing_team_id.as_deref() }
    pub fn get_nr_of_dice(&self) -> i32 { self.nr_of_dice }
    pub fn get_block_roll(&self) -> &[i32] { &self.block_roll }
    pub fn has_team_re_roll_option(&self) -> bool { self.team_re_roll_option }
    pub fn has_pro_re_roll_option(&self) -> bool { self.pro_re_roll_option }
}

impl IDialogParameter for DialogBlockRollParameter {
    fn get_id(&self) -> DialogId { DialogId::BLOCK_ROLL }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_round_trip() {
        let p = DialogBlockRollParameter {
            choosing_team_id: Some("teamA".into()),
            nr_of_dice: 2,
            block_roll: vec![3, 5],
            team_re_roll_option: true,
            pro_re_roll_option: false,
        };
        let json = serde_json::to_string(&p).unwrap();
        let back: DialogBlockRollParameter = serde_json::from_str(&json).unwrap();
        assert_eq!(back.get_choosing_team_id(), Some("teamA"));
        assert_eq!(back.get_nr_of_dice(), 2);
        assert_eq!(back.get_block_roll(), &[3, 5]);
        assert!(back.has_team_re_roll_option());
        assert!(!back.has_pro_re_roll_option());
    }

    #[test]
    fn get_id_is_block_roll() {
        assert_eq!(DialogBlockRollParameter::default().get_id(), DialogId::BLOCK_ROLL);
    }

    #[test]
    fn default_is_sensible() {
        let p = DialogBlockRollParameter::default();
        assert!(p.get_choosing_team_id().is_none());
        assert_eq!(p.get_nr_of_dice(), 0);
        assert!(p.get_block_roll().is_empty());
        assert!(!p.has_team_re_roll_option());
        assert!(!p.has_pro_re_roll_option());
    }

    #[test]
    fn accessor_methods_with_non_default_values() {
        let p = DialogBlockRollParameter {
            choosing_team_id: Some("home".into()),
            nr_of_dice: 3,
            block_roll: vec![1, 4, 6],
            team_re_roll_option: true,
            pro_re_roll_option: true,
        };
        assert_eq!(p.get_choosing_team_id(), Some("home"));
        assert_eq!(p.get_nr_of_dice(), 3);
        assert_eq!(p.get_block_roll(), &[1, 4, 6]);
        assert!(p.has_team_re_roll_option());
        assert!(p.has_pro_re_roll_option());
    }

    #[test]
    fn none_choosing_team_id_is_edge_case() {
        let p = DialogBlockRollParameter {
            choosing_team_id: None,
            nr_of_dice: 1,
            block_roll: vec![2],
            team_re_roll_option: false,
            pro_re_roll_option: false,
        };
        assert!(p.get_choosing_team_id().is_none());
    }
}
