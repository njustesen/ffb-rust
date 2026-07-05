use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogSelectPositionParameter.
/// Note: PositionChoiceMode serialized as String (stub not yet translated).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogSelectPositionParameter {
    pub positions: Vec<String>,
    pub position_choice_mode: Option<String>,
    pub min_select: i32,
    pub max_select: i32,
    pub team_id: Option<String>,
}

impl DialogSelectPositionParameter {
    pub fn get_positions(&self) -> &[String] { &self.positions }
    pub fn get_position_choice_mode(&self) -> Option<&str> { self.position_choice_mode.as_deref() }
    pub fn get_min_select(&self) -> i32 { self.min_select }
    pub fn get_max_select(&self) -> i32 { self.max_select }
    pub fn get_team_id(&self) -> Option<&str> { self.team_id.as_deref() }
    pub fn add_position(&mut self, pos: impl Into<String>) {
        let s = pos.into();
        if !s.is_empty() { self.positions.push(s); }
    }
}

impl IDialogParameter for DialogSelectPositionParameter {
    fn get_id(&self) -> DialogId { DialogId::SELECT_POSITION }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dialog_id_is_select_position() {
        assert_eq!(DialogSelectPositionParameter::default().get_id(), DialogId::SELECT_POSITION);
    }
    #[test]
    fn add_position_filters_empty_string() {
        let mut p = DialogSelectPositionParameter::default();
        p.add_position("pos1");
        p.add_position("");
        assert_eq!(p.get_positions().len(), 1);
    }
}
