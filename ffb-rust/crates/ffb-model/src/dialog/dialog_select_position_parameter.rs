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

    #[test]
    fn default_is_sensible() {
        let p = DialogSelectPositionParameter::default();
        assert!(p.get_positions().is_empty());
        assert!(p.get_position_choice_mode().is_none());
        assert_eq!(p.get_min_select(), 0);
        assert_eq!(p.get_max_select(), 0);
        assert!(p.get_team_id().is_none());
    }

    #[test]
    fn team_id_and_min_max_select() {
        let p = DialogSelectPositionParameter {
            team_id: Some("home".into()),
            min_select: 1,
            max_select: 4,
            ..Default::default()
        };
        assert_eq!(p.get_team_id(), Some("home"));
        assert_eq!(p.get_min_select(), 1);
        assert_eq!(p.get_max_select(), 4);
    }

    #[test]
    fn position_choice_mode_accessor() {
        let p = DialogSelectPositionParameter {
            position_choice_mode: Some("SINGLE".into()),
            ..Default::default()
        };
        assert_eq!(p.get_position_choice_mode(), Some("SINGLE"));
    }
}
