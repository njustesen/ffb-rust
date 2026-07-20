use serde::{Deserialize, Serialize};
use crate::enums::Direction;
use crate::types::FieldCoordinate;

/// One candidate landing square for a pushed player.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PushbackSquare {
    pub coordinate: FieldCoordinate,
    pub direction: Direction,
    /// Whether this square belongs to the home team's pushback choice set.
    pub home_choice: bool,
    pub selected: bool,
    pub locked: bool,
}

impl PushbackSquare {
    pub fn new(coordinate: FieldCoordinate, direction: Direction, home_choice: bool) -> Self {
        PushbackSquare { coordinate, direction, home_choice, selected: false, locked: false }
    }

    pub fn transform(self) -> PushbackSquare {
        PushbackSquare {
            coordinate: self.coordinate.transform(),
            direction: self.direction.transform(),
            home_choice: !self.home_choice,
            selected: self.selected,
            locked: self.locked,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::Direction;
    use crate::types::FieldCoordinate;

    #[test]
    fn pushback_square_home_choice_flips_on_transform() {
        let sq = PushbackSquare::new(FieldCoordinate::new(10, 7), Direction::East, true);
        let t = sq.transform();
        assert!(!t.home_choice);
        assert_eq!(t.coordinate.x, crate::types::field_coordinate::FIELD_WIDTH - 1 - 10);
    }

    #[test]
    fn pushback_square_transform_mirrors_direction() {
        let sq = PushbackSquare::new(FieldCoordinate::new(10, 7), Direction::East, true);
        let t = sq.transform();
        assert_eq!(t.direction, Direction::West);
    }

    #[test]
    fn serde_round_trip() {
        let sq = PushbackSquare::new(FieldCoordinate::new(5, 5), Direction::North, false);
        let json = serde_json::to_string(&sq).unwrap();
        let back: PushbackSquare = serde_json::from_str(&json).unwrap();
        assert_eq!(sq, back);
    }

    #[test]
    fn new_starts_unselected_and_unlocked() {
        let sq = PushbackSquare::new(FieldCoordinate::new(3, 3), Direction::South, true);
        assert!(!sq.selected);
        assert!(!sq.locked);
        assert!(sq.home_choice);
    }

    #[test]
    fn transform_double_inverts_home_choice() {
        let sq = PushbackSquare::new(FieldCoordinate::new(10, 7), Direction::East, true);
        let t = sq.transform().transform();
        assert_eq!(t.home_choice, sq.home_choice);
    }

    #[test]
    fn copy_semantics() {
        let sq = PushbackSquare::new(FieldCoordinate::new(1, 1), Direction::North, false);
        let sq2 = sq;
        assert_eq!(sq, sq2);
    }
}
