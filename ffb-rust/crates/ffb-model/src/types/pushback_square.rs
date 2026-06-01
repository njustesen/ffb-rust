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
    fn transform_flips_side() {
        let sq = PushbackSquare::new(FieldCoordinate::new(10, 7), Direction::East, true);
        let t = sq.transform();
        assert_eq!(t.home_choice, false);
        assert_eq!(t.direction, Direction::West);
        assert_eq!(t.coordinate.x, 25 - 10);
    }

    #[test]
    fn serde_round_trip() {
        let sq = PushbackSquare::new(FieldCoordinate::new(5, 5), Direction::North, false);
        let json = serde_json::to_string(&sq).unwrap();
        let back: PushbackSquare = serde_json::from_str(&json).unwrap();
        assert_eq!(sq, back);
    }
}
