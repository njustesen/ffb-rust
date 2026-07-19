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
    #[test]
    fn dialog_id_is_kick_skill() {
        assert_eq!(DialogKickSkillParameter::default().get_id(), DialogId::KICK_SKILL);
    }
    #[test]
    fn stores_player_id_and_ball_coordinate() {
        let p = DialogKickSkillParameter {
            player_id: Some("p1".into()),
            ball_coordinate: Some(FieldCoordinate::new(3, 5)),
            ..Default::default()
        };
        assert_eq!(p.get_player_id(), Some("p1"));
        assert_eq!(p.get_ball_coordinate(), Some(FieldCoordinate::new(3, 5)));
    }

    #[test]
    fn default_is_sensible() {
        let p = DialogKickSkillParameter::default();
        assert!(p.get_player_id().is_none());
        assert!(p.get_ball_coordinate().is_none());
        assert!(p.get_ball_coordinate_with_kick().is_none());
    }

    #[test]
    fn accessor_methods_with_non_default_values() {
        let p = DialogKickSkillParameter {
            player_id: Some("kicker1".into()),
            ball_coordinate: Some(FieldCoordinate::new(7, 3)),
            ball_coordinate_with_kick: Some(FieldCoordinate::new(10, 3)),
        };
        assert_eq!(p.get_player_id(), Some("kicker1"));
        assert_eq!(p.get_ball_coordinate(), Some(FieldCoordinate::new(7, 3)));
        assert_eq!(p.get_ball_coordinate_with_kick(), Some(FieldCoordinate::new(10, 3)));
    }

    #[test]
    fn none_ball_coordinate_with_kick_is_edge_case() {
        let p = DialogKickSkillParameter {
            player_id: Some("p2".into()),
            ball_coordinate: Some(FieldCoordinate::new(5, 5)),
            ball_coordinate_with_kick: None,
        };
        assert!(p.get_ball_coordinate_with_kick().is_none());
        assert!(p.get_ball_coordinate().is_some());
    }

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
