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

    /// Java: `MoveSquare.toJsonValue()`.
    pub fn to_json_value(&self) -> serde_json::Value {
        use crate::util::java_json as j;
        let mut o = j::JsonObject::new();
        o.insert("coordinate".into(), j::field_coordinate_or_null(Some(self.coordinate)));
        o.insert("minimumRollDodge".into(), serde_json::Value::from(self.minimum_roll_dodge));
        o.insert("minimumRollGfi".into(), serde_json::Value::from(self.minimum_roll_gfi));
        serde_json::Value::Object(o)
    }

    /// Java: `new MoveSquare().initFrom(source, jsonValue)`. A null coordinate is `Err`: the only
    /// Java caller adds the square to a `HashSet`, whose `hashCode()` (`getCoordinate().hashCode()`)
    /// then throws.
    pub fn init_from(json: &serde_json::Value) -> Result<Self, String> {
        use crate::util::java_json as j;
        let o = j::to_json_object(json)?;
        let coordinate = j::get_field_coordinate(o, "coordinate")?.ok_or("MoveSquare: null coordinate (hashCode NPE)")?;
        Ok(MoveSquare {
            coordinate,
            minimum_roll_dodge: j::get_int(o, "minimumRollDodge")?,
            minimum_roll_gfi: j::get_int(o, "minimumRollGfi")?,
        })
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

    #[test]
    fn move_square_is_dodging_when_dodge_roll_positive() {
        let sq = MoveSquare::new(FieldCoordinate::new(5, 5), 3, 0);
        assert!(sq.is_dodging());
    }

    #[test]
    fn move_square_not_dodging_when_roll_zero() {
        let sq = MoveSquare::new(FieldCoordinate::new(5, 5), 0, 0);
        assert!(!sq.is_dodging());
    }

    #[test]
    fn move_square_is_going_for_it_when_gfi_roll_positive() {
        let sq = MoveSquare::new(FieldCoordinate::new(5, 5), 0, 2);
        assert!(sq.is_going_for_it());
        assert!(!sq.is_dodging());
    }

    #[test]
    fn transform_preserves_rolls() {
        let sq = MoveSquare::new(FieldCoordinate::new(8, 3), 4, 2);
        let t = sq.transform();
        // coordinate should be mirrored but rolls must be unchanged
        assert_eq!(t.minimum_roll_dodge, 4);
        assert_eq!(t.minimum_roll_gfi, 2);
        // y coordinate is untouched by transform
        assert_eq!(t.coordinate.y, 3);
    }

}
