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

impl DialogKickSkillParameter {
    /// Mirrors Java's `transform()`: player id is passed through unchanged, but both
    /// ball coordinates are mirrored via `FieldCoordinate.transform()`.
    fn transform_typed(&self) -> DialogKickSkillParameter {
        DialogKickSkillParameter {
            player_id: self.player_id.clone(),
            ball_coordinate: self.ball_coordinate.map(|c| c.transform()),
            ball_coordinate_with_kick: self.ball_coordinate_with_kick.map(|c| c.transform()),
        }
    }
}

impl IDialogParameter for DialogKickSkillParameter {
    fn get_id(&self) -> DialogId { DialogId::KICK_SKILL }
    fn transform(&self) -> Box<dyn IDialogParameter> {
        // Java: FieldCoordinate.transform(getBallCoordinate()) / (getBallCoordinateWithKick())
        // mirrors the coordinates to the other side of the field; a naive clone would keep
        // them on the original (now-wrong) side after possession/side transformation.
        Box::new(self.transform_typed())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::FieldCoordinate;
/// Java's `transform()` calls `FieldCoordinate.transform(getBallCoordinate())`, which
    /// mirrors the coordinate across the field (used when the dialog's side/possession
    /// flips). A naive `self.clone()` — the pre-fix Rust behavior — left the coordinates
    /// untouched, which is wrong once the field is mirrored. This test fails against that
    /// naive-clone behavior and passes once transform() actually mirrors the coordinates.
    #[test]
    fn transform_mirrors_ball_coordinates_not_naive_clone() {
        let original = FieldCoordinate::new(5, 5);
        let with_kick = FieldCoordinate::new(10, 8);
        let p = DialogKickSkillParameter {
            player_id: Some("kicker".into()),
            ball_coordinate: Some(original),
            ball_coordinate_with_kick: Some(with_kick),
        };
        let transformed = p.transform_typed();

        // player_id passes through unchanged.
        assert_eq!(transformed.get_player_id(), Some("kicker"));

        // Coordinates must be mirrored (FieldCoordinate::transform), not identical to
        // the originals — a naive clone would have kept them equal to `original`/`with_kick`.
        assert_eq!(transformed.get_ball_coordinate(), Some(original.transform()));
        assert_ne!(transformed.get_ball_coordinate(), Some(original));
        assert_eq!(transformed.get_ball_coordinate_with_kick(), Some(with_kick.transform()));
        assert_ne!(transformed.get_ball_coordinate_with_kick(), Some(with_kick));
    }

    #[test]
    fn transform_none_coordinates_stay_none() {
        let p = DialogKickSkillParameter {
            player_id: Some("kicker".into()),
            ball_coordinate: None,
            ball_coordinate_with_kick: None,
        };
        let transformed = p.transform_typed();
        assert!(transformed.get_ball_coordinate().is_none());
        assert!(transformed.get_ball_coordinate_with_kick().is_none());
    }
}
