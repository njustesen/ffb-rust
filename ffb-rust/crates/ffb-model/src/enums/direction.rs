use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum Direction {
    North,
    Northeast,
    East,
    Southeast,
    South,
    Southwest,
    West,
    Northwest,
}

impl Direction {
    pub fn name(self) -> &'static str {
        match self {
            Direction::North => "North",
            Direction::Northeast => "Northeast",
            Direction::East => "East",
            Direction::Southeast => "Southeast",
            Direction::South => "South",
            Direction::Southwest => "Southwest",
            Direction::West => "West",
            Direction::Northwest => "Northwest",
        }
    }

    pub fn from_name(name: &str) -> Option<Direction> {
        match name.to_lowercase().as_str() {
            "north" => Some(Direction::North),
            "northeast" => Some(Direction::Northeast),
            "east" => Some(Direction::East),
            "southeast" => Some(Direction::Southeast),
            "south" => Some(Direction::South),
            "southwest" => Some(Direction::Southwest),
            "west" => Some(Direction::West),
            "northwest" => Some(Direction::Northwest),
            _ => None,
        }
    }

    /// Mirror horizontally (flip left/right)
    pub fn transform(self) -> Direction {
        match self {
            Direction::Northeast => Direction::Northwest,
            Direction::East => Direction::West,
            Direction::Southeast => Direction::Southwest,
            Direction::Southwest => Direction::Southeast,
            Direction::West => Direction::East,
            Direction::Northwest => Direction::Northeast,
            other => other,
        }
    }

    /// dx component for this direction (-1, 0, or 1)
    pub fn dx(self) -> i8 {
        match self {
            Direction::North | Direction::South => 0,
            Direction::Northeast | Direction::East | Direction::Southeast => 1,
            Direction::Northwest | Direction::West | Direction::Southwest => -1,
        }
    }

    /// dy component for this direction (-1, 0, or 1; negative = north)
    pub fn dy(self) -> i8 {
        match self {
            Direction::North | Direction::Northeast | Direction::Northwest => -1,
            Direction::South | Direction::Southeast | Direction::Southwest => 1,
            Direction::East | Direction::West => 0,
        }
    }

    /// Map a D8 roll (1–8) to a scatter direction. Mirrors Java DirectionFactory.forRoll().
    pub fn for_roll(roll: i32) -> Option<Direction> {
        match roll {
            1 => Some(Direction::North),
            2 => Some(Direction::Northeast),
            3 => Some(Direction::East),
            4 => Some(Direction::Southeast),
            5 => Some(Direction::South),
            6 => Some(Direction::Southwest),
            7 => Some(Direction::West),
            8 => Some(Direction::Northwest),
            _ => None,
        }
    }

    /// Compute the direction from one coordinate to another (Java: FieldCoordinate.getDirection).
    pub fn from_coords(from_x: i32, from_y: i32, to_x: i32, to_y: i32) -> Option<Direction> {
        let dx = to_x - from_x;
        let dy = to_y - from_y;
        match (dx.signum(), dy.signum()) {
            (-1, -1) => Some(Direction::Northwest),
            (-1,  1) => Some(Direction::Southwest),
            (-1,  0) => Some(Direction::West),
            ( 1, -1) => Some(Direction::Northeast),
            ( 1,  1) => Some(Direction::Southeast),
            ( 1,  0) => Some(Direction::East),
            ( 0, -1) => Some(Direction::North),
            ( 0,  1) => Some(Direction::South),
            _ => None,
        }
    }

    pub fn all() -> &'static [Direction] {
        &[
            Direction::North,
            Direction::Northeast,
            Direction::East,
            Direction::Southeast,
            Direction::South,
            Direction::Southwest,
            Direction::West,
            Direction::Northwest,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn for_roll_all_eight_faces() {
        assert_eq!(Direction::for_roll(1), Some(Direction::North));
        assert_eq!(Direction::for_roll(2), Some(Direction::Northeast));
        assert_eq!(Direction::for_roll(3), Some(Direction::East));
        assert_eq!(Direction::for_roll(4), Some(Direction::Southeast));
        assert_eq!(Direction::for_roll(5), Some(Direction::South));
        assert_eq!(Direction::for_roll(6), Some(Direction::Southwest));
        assert_eq!(Direction::for_roll(7), Some(Direction::West));
        assert_eq!(Direction::for_roll(8), Some(Direction::Northwest));
    }

    #[test]
    fn for_roll_out_of_range_returns_none() {
        assert_eq!(Direction::for_roll(0), None);
        assert_eq!(Direction::for_roll(9), None);
    }

    #[test]
    fn round_trip_name() {
        for d in Direction::all() {
            assert_eq!(Direction::from_name(d.name()), Some(*d));
        }
    }

    #[test]
    fn transform_is_mirror() {
        assert_eq!(Direction::Northeast.transform(), Direction::Northwest);
        assert_eq!(Direction::East.transform(), Direction::West);
        assert_eq!(Direction::Southeast.transform(), Direction::Southwest);
        assert_eq!(Direction::North.transform(), Direction::North);
        assert_eq!(Direction::South.transform(), Direction::South);
        for d in Direction::all() {
            assert_eq!(d.transform().transform(), *d);
        }
    }

    #[test]
    fn dx_dy_are_unit() {
        for d in Direction::all() {
            let dx = d.dx() as i32;
            let dy = d.dy() as i32;
            assert!(dx.abs() <= 1 && dy.abs() <= 1);
            assert!(dx != 0 || dy != 0, "direction {:?} has zero vector", d);
        }
    }

    #[test]
    fn serde_round_trip() {
        let d = Direction::Southeast;
        let json = serde_json::to_string(&d).unwrap();
        let back: Direction = serde_json::from_str(&json).unwrap();
        assert_eq!(d, back);
    }

    #[test]
    fn count_is_8() {
        assert_eq!(Direction::all().len(), 8);
    }

    #[test]
    fn all_have_non_empty_names() {
        for d in Direction::all() {
            assert!(!d.name().is_empty());
        }
    }

    #[test]
    fn north_name_is_north() {
        assert_eq!(Direction::North.name(), "North");
    }

    #[test]
    fn south_name_is_south() {
        assert_eq!(Direction::South.name(), "South");
    }

    #[test]
    fn east_name_is_east() {
        assert_eq!(Direction::East.name(), "East");
    }

    #[test]
    fn from_name_case_insensitive() {
        assert_eq!(Direction::from_name("north"), Some(Direction::North));
        assert_eq!(Direction::from_name("SOUTH"), Some(Direction::South));
    }

    #[test]
    fn from_name_unknown_is_none() {
        assert_eq!(Direction::from_name("unknown"), None);
    }
}
