use serde::{Deserialize, Serialize};
use crate::types::FieldCoordinate;
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogKickSkillParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogKickSkillParameter {
    pub player_id: Option<String>,
    pub ball_coordinate: Option<FieldCoordinate>,
    pub ball_coordinate_with_kick: Option<FieldCoordinate>,
}

impl DialogKickSkillParameter {
    pub fn get_player_id(&self) -> Option<&str> { self.player_id.as_deref() }
    pub fn get_ball_coordinate(&self) -> Option<FieldCoordinate> { self.ball_coordinate }
    pub fn get_ball_coordinate_with_kick(&self) -> Option<FieldCoordinate> { self.ball_coordinate_with_kick }
}

impl IDialogParameter for DialogKickSkillParameter {
    fn get_id(&self) -> DialogId { DialogId::KICK_SKILL }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}
