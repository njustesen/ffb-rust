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

/// `(DialogId) IJsonOption.DIALOG_ID.getFrom(source, jsonObject)`: `JsonEnumWithNameOption` over
/// `DialogIdFactory` -- absent or JSON null -> null, else `forName` (case-insensitive; unknown -> null).
pub fn get_dialog_id(obj: &crate::util::java_json::JsonObject) -> Result<Option<DialogId>, String> {
    Ok(crate::util::java_json::get_enum_name(obj, "dialogId")?.and_then(DialogId::for_name_ignore_case))
}

/// `UtilDialogParameter.validateDialogId(this, DIALOG_ID.getFrom(source, jsonObject))` inside a
/// parameter's `initFrom`: a different (or null) received id throws IllegalStateException.
pub fn validate_dialog_id_json(expected: DialogId, obj: &crate::util::java_json::JsonObject) -> Result<(), String> {
    let received = get_dialog_id(obj)?;
    if received != Some(expected) {
        return Err(format!(
            "Wrong dialog id. Expected {} received {}",
            expected.get_name(),
            received.map(DialogId::get_name).unwrap_or("null")
        ));
    }
    Ok(())
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
    #[test]
    fn validate_block_roll_passes() {
        let param = DialogBlockRollParameter { choosing_team_id: Some("t1".into()), nr_of_dice: 2, ..Default::default() };
        UtilDialogParameter::validate_dialog_id(&param, DialogId::BLOCK_ROLL);
    }
}
