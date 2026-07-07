use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.UtilDialogParameter.
pub struct UtilDialogParameter;

impl UtilDialogParameter {
    pub fn validate_dialog_id(dialog_parameter: &dyn IDialogParameter, received_id: DialogId) {
        if dialog_parameter.get_id() != received_id {
            panic!(
                "Wrong dialog id. Expected {} received {}",
                dialog_parameter.get_id().get_name(),
                received_id.get_name()
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::dialog_re_roll_parameter::DialogReRollParameter;
    use super::super::dialog_block_roll_parameter::DialogBlockRollParameter;

    #[test]
    fn validate_matching_id_passes() {
        let param = DialogReRollParameter { player_id: Some("p1".into()), minimum_roll: 3, ..Default::default() };
        UtilDialogParameter::validate_dialog_id(&param, DialogId::RE_ROLL);
    }

    #[test]
    #[should_panic(expected = "Wrong dialog id")]
    fn validate_mismatched_id_panics() {
        let param = DialogBlockRollParameter { choosing_team_id: Some("t1".into()), nr_of_dice: 2, ..Default::default() };
        UtilDialogParameter::validate_dialog_id(&param, DialogId::RE_ROLL);
    }

    #[test]
    fn validate_block_roll_with_block_roll_id_passes() {
        let param = DialogBlockRollParameter { choosing_team_id: Some("t2".into()), nr_of_dice: 3, ..Default::default() };
        UtilDialogParameter::validate_dialog_id(&param, DialogId::BLOCK_ROLL);
    }

    #[test]
    #[should_panic(expected = "Wrong dialog id")]
    fn validate_re_roll_with_block_roll_id_panics() {
        let param = DialogReRollParameter { player_id: Some("p2".into()), minimum_roll: 4, ..Default::default() };
        UtilDialogParameter::validate_dialog_id(&param, DialogId::BLOCK_ROLL);
    }
}
