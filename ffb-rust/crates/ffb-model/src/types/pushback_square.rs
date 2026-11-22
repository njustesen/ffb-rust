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

    /// Java: `PushbackSquare.toJsonValue()`.
    pub fn to_json_value(&self) -> serde_json::Value {
        use crate::util::java_json as j;
        let mut o = j::JsonObject::new();
        o.insert("coordinate".into(), j::field_coordinate_or_null(Some(self.coordinate)));
        o.insert("direction".into(), serde_json::Value::from(self.direction.name()));
        o.insert("selected".into(), serde_json::Value::Bool(self.selected));
        o.insert("locked".into(), serde_json::Value::Bool(self.locked));
        o.insert("homeChoice".into(), serde_json::Value::Bool(self.home_choice));
        serde_json::Value::Object(o)
    }

    /// Java: `new PushbackSquare().initFrom(source, jsonValue)`. `Err` for a null coordinate (the
    /// `HashSet` it goes into hashes the coordinate -> NPE) and for a null/unknown direction
    /// (`DirectionFactory.forName` null, which the Rust struct cannot hold); the three booleans are
    /// primitives (a missing one is an unboxing NPE).
    pub fn init_from(json: &serde_json::Value) -> Result<Self, String> {
        use crate::util::java_json as j;
        let o = j::to_json_object(json)?;
        let coordinate =
            j::get_field_coordinate(o, "coordinate")?.ok_or("PushbackSquare: null coordinate (hashCode NPE)")?;
        let name = j::get_enum_name(o, "direction")?;
        let direction = name
            .and_then(Direction::for_name_ignore_case)
            .ok_or_else(|| format!("PushbackSquare: direction {name:?} is null in Java"))?;
        Ok(PushbackSquare {
            coordinate,
            direction,
            selected: j::get_boolean(o, "selected")?,
            locked: j::get_boolean(o, "locked")?,
            home_choice: j::get_boolean(o, "homeChoice")?,
        })
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
    fn transform_double_inverts_home_choice() {
        let sq = PushbackSquare::new(FieldCoordinate::new(10, 7), Direction::East, true);
        let t = sq.transform().transform();
        assert_eq!(t.home_choice, sq.home_choice);
    }

}
