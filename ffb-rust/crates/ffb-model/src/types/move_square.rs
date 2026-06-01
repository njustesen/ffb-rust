use serde::{Deserialize, Serialize};
use crate::types::FieldCoordinate;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MoveSquareKind {
    Move,
    Dodge,
    Rush,
    RushDodge,
}

/// A reachable square during a player's move, recording the rolls needed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MoveSquare {
    pub coordinate: FieldCoordinate,
    /// Minimum d6 needed to dodge into this square (0 = no dodge required).
    pub minimum_roll_dodge: i32,
    /// Minimum d6 needed for a go-for-it into this square (0 = not needed).
    pub minimum_roll_gfi: i32,
}

impl MoveSquare {
    pub fn new(coordinate: FieldCoordinate, minimum_roll_dodge: i32, minimum_roll_gfi: i32) -> Self {
        MoveSquare { coordinate, minimum_roll_dodge, minimum_roll_gfi }
    }

    pub fn is_dodging(self) -> bool {
        self.minimum_roll_dodge > 0
    }

    pub fn is_going_for_it(self) -> bool {
        self.minimum_roll_gfi > 0
    }

    pub fn kind(self) -> MoveSquareKind {
        match (self.is_dodging(), self.is_going_for_it()) {
            (false, false) => MoveSquareKind::Move,
            (true, false) => MoveSquareKind::Dodge,
            (false, true) => MoveSquareKind::Rush,
            (true, true) => MoveSquareKind::RushDodge,
        }
    }

    pub fn transform(self) -> MoveSquare {
        MoveSquare { coordinate: self.coordinate.transform(), ..self }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::FieldCoordinate;

    #[test]
    fn move_square_kind() {
        let c = FieldCoordinate::new(5, 5);
        assert_eq!(MoveSquare::new(c, 0, 0).kind(), MoveSquareKind::Move);
        assert_eq!(MoveSquare::new(c, 3, 0).kind(), MoveSquareKind::Dodge);
        assert_eq!(MoveSquare::new(c, 0, 2).kind(), MoveSquareKind::Rush);
        assert_eq!(MoveSquare::new(c, 3, 2).kind(), MoveSquareKind::RushDodge);
    }

    #[test]
    fn transform_mirrors_coordinate() {
        let sq = MoveSquare::new(FieldCoordinate::new(10, 7), 3, 0);
        let t = sq.transform();
        assert_eq!(t.coordinate.x, 25 - 10);
        assert_eq!(t.minimum_roll_dodge, 3);
    }

    #[test]
    fn serde_round_trip() {
        let sq = MoveSquare::new(FieldCoordinate::new(3, 5), 2, 0);
        let json = serde_json::to_string(&sq).unwrap();
        let back: MoveSquare = serde_json::from_str(&json).unwrap();
        assert_eq!(sq, back);
    }
}
