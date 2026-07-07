use serde::{Deserialize, Serialize};
use crate::types::FieldCoordinate;

/// Pass/throw range ruler showing the minimum roll needed for the current throw.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RangeRuler {
    pub thrower_id: String,
    pub target_coordinate: Option<FieldCoordinate>,
    /// Raw minimum d6 value; negative = out of range, 0 = no roll needed.
    pub minimum_roll: i32,
    pub throw_team_mate: bool,
}

impl RangeRuler {
    pub fn new(
        thrower_id: String,
        target_coordinate: Option<FieldCoordinate>,
        minimum_roll: i32,
        throw_team_mate: bool,
    ) -> Self {
        RangeRuler { thrower_id, target_coordinate, minimum_roll, throw_team_mate }
    }

    /// Human-readable roll string matching the Java implementation.
    pub fn minimum_roll_display(&self) -> &'static str {
        if self.minimum_roll == 0 {
            "--"
        } else if self.minimum_roll < 0 {
            ""
        } else if self.minimum_roll < 6 {
            match self.minimum_roll {
                2 => "2+",
                3 => "3+",
                4 => "4+",
                5 => "5+",
                _ => "?+",
            }
        } else {
            "6"
        }
    }

    pub fn transform(self) -> RangeRuler {
        RangeRuler {
            target_coordinate: self.target_coordinate.map(|c| c.transform()),
            ..self
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::FieldCoordinate;

    #[test]
    fn display_strings() {
        let make = |roll| RangeRuler::new("p1".into(), None, roll, false);
        assert_eq!(make(0).minimum_roll_display(), "--");
        assert_eq!(make(-1).minimum_roll_display(), "");
        assert_eq!(make(3).minimum_roll_display(), "3+");
        assert_eq!(make(6).minimum_roll_display(), "6");
    }

    #[test]
    fn transform_mirrors_target() {
        let r = RangeRuler::new(
            "p1".into(),
            Some(FieldCoordinate::new(10, 7)),
            3,
            false,
        );
        let t = r.transform();
        assert_eq!(t.target_coordinate.unwrap().x, 25 - 10);
        assert_eq!(t.minimum_roll, 3);
    }

    #[test]
    fn serde_round_trip() {
        let r = RangeRuler::new(
            "p1".into(),
            Some(FieldCoordinate::new(13, 7)),
            4,
            false,
        );
        let json = serde_json::to_string(&r).unwrap();
        let back: RangeRuler = serde_json::from_str(&json).unwrap();
        assert_eq!(r, back);
    }

    #[test]
    fn display_strings_all_in_range_values() {
        let make = |roll| RangeRuler::new("p1".into(), None, roll, false);
        // Every explicit in-range variant
        assert_eq!(make(2).minimum_roll_display(), "2+");
        assert_eq!(make(4).minimum_roll_display(), "4+");
        assert_eq!(make(5).minimum_roll_display(), "5+");
        // An unlisted in-range value falls through to "?+"
        assert_eq!(make(1).minimum_roll_display(), "?+");
    }

    #[test]
    fn transform_with_no_target_coordinate() {
        let r = RangeRuler::new("p2".into(), None, 5, true);
        let t = r.transform();
        // None target should remain None after transform
        assert!(t.target_coordinate.is_none());
        // other fields preserved
        assert_eq!(t.thrower_id, "p2");
        assert_eq!(t.minimum_roll, 5);
        assert!(t.throw_team_mate);
    }
}
