use serde::{Deserialize, Serialize};
use crate::enums::Direction;

// ── Dugout X coordinates (negative = home side, 30+ = away side) ─────────────
pub const FIELD_WIDTH: i32 = 26;
pub const FIELD_HEIGHT: i32 = 15;

pub const RSV_HOME_X: i32 = -1;
pub const KO_HOME_X: i32 = -2;
pub const BH_HOME_X: i32 = -3;
pub const SI_HOME_X: i32 = -4;
pub const RIP_HOME_X: i32 = -5;
pub const BAN_HOME_X: i32 = -6;
pub const MNG_HOME_X: i32 = -7;

pub const RSV_AWAY_X: i32 = 30;
pub const KO_AWAY_X: i32 = 31;
pub const BH_AWAY_X: i32 = 32;
pub const SI_AWAY_X: i32 = 33;
pub const RIP_AWAY_X: i32 = 34;
pub const BAN_AWAY_X: i32 = 35;
pub const MNG_AWAY_X: i32 = 36;

pub const SWEET_SPOT_AWAY: FieldCoordinate = FieldCoordinate { x: 19, y: 7 };
pub const SWEET_SPOT_HOME: FieldCoordinate = FieldCoordinate { x: 6, y: 7 };

/// A position on or off the pitch (dugout slots use special x values).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FieldCoordinate {
    pub x: i32,
    pub y: i32,
}

impl FieldCoordinate {
    pub const fn new(x: i32, y: i32) -> Self {
        FieldCoordinate { x, y }
    }

    pub fn add(self, dx: i32, dy: i32) -> FieldCoordinate {
        FieldCoordinate { x: self.x + dx, y: self.y + dy }
    }

    /// Chebyshev distance (max of abs dx, abs dy).
    pub fn distance_in_steps(self, other: FieldCoordinate) -> i32 {
        (self.x - other.x).abs().max((self.y - other.y).abs())
    }

    pub fn is_adjacent(self, other: FieldCoordinate) -> bool {
        self.distance_in_steps(other) == 1
    }

    /// True if this is a normal on-pitch square (not a dugout slot).
    pub fn is_on_pitch(self) -> bool {
        self.x >= 0 && self.x < FIELD_WIDTH && self.y >= 0 && self.y < FIELD_HEIGHT
    }

    pub fn is_box_coordinate(self) -> bool {
        matches!(
            self.x,
            RSV_HOME_X | RSV_AWAY_X | KO_HOME_X | KO_AWAY_X | BH_HOME_X | BH_AWAY_X
                | SI_HOME_X | SI_AWAY_X | RIP_HOME_X | RIP_AWAY_X | BAN_HOME_X | BAN_AWAY_X
                | MNG_HOME_X | MNG_AWAY_X
        )
    }

    /// Mirror across the vertical centre-line (used when possession changes sides).
    pub fn transform(self) -> FieldCoordinate {
        let new_x = match self.x {
            RSV_HOME_X => RSV_AWAY_X,
            KO_HOME_X => KO_AWAY_X,
            BH_HOME_X => BH_AWAY_X,
            SI_HOME_X => SI_AWAY_X,
            RIP_HOME_X => RIP_AWAY_X,
            BAN_HOME_X => BAN_AWAY_X,
            MNG_HOME_X => MNG_AWAY_X,
            RSV_AWAY_X => RSV_HOME_X,
            KO_AWAY_X => KO_HOME_X,
            BH_AWAY_X => BH_HOME_X,
            SI_AWAY_X => SI_HOME_X,
            RIP_AWAY_X => RIP_HOME_X,
            BAN_AWAY_X => BAN_HOME_X,
            MNG_AWAY_X => MNG_HOME_X,
            x => FIELD_WIDTH - 1 - x,
        };
        FieldCoordinate { x: new_x, y: self.y }
    }

    /// Direction from `self` to `to`; returns None if they are the same square.
    pub fn direction_to(self, to: FieldCoordinate) -> Option<Direction> {
        let dx = to.x - self.x;
        let dy = to.y - self.y;
        match (dx.signum(), dy.signum()) {
            (0, 0) => None,
            (-1, -1) => Some(Direction::Northwest),
            (-1, 0) => Some(Direction::West),
            (-1, 1) => Some(Direction::Southwest),
            (0, -1) => Some(Direction::North),
            (0, 1) => Some(Direction::South),
            (1, -1) => Some(Direction::Northeast),
            (1, 0) => Some(Direction::East),
            (1, 1) => Some(Direction::Southeast),
            _ => unreachable!(),
        }
    }

    /// Step `distance` squares in direction `d`.
    pub fn step(self, d: Direction, distance: i32) -> FieldCoordinate {
        FieldCoordinate {
            x: self.x + d.dx() as i32 * distance,
            y: self.y + d.dy() as i32 * distance,
        }
    }

    /// All 8 adjacent squares (may be off-pitch).
    pub fn neighbours(self) -> [FieldCoordinate; 8] {
        [
            self.step(Direction::North, 1),
            self.step(Direction::Northeast, 1),
            self.step(Direction::East, 1),
            self.step(Direction::Southeast, 1),
            self.step(Direction::South, 1),
            self.step(Direction::Southwest, 1),
            self.step(Direction::West, 1),
            self.step(Direction::Northwest, 1),
        ]
    }
}

impl std::fmt::Display for FieldCoordinate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({},{})", self.x, self.y)
    }
}

// ── FieldCoordinateBounds ─────────────────────────────────────────────────────

/// An axis-aligned rectangle of field squares.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FieldCoordinateBounds {
    pub top_left: FieldCoordinate,
    pub bottom_right: FieldCoordinate,
}

impl FieldCoordinateBounds {
    pub const FIELD: FieldCoordinateBounds = FieldCoordinateBounds {
        top_left: FieldCoordinate::new(0, 0),
        bottom_right: FieldCoordinate::new(25, 14),
    };
    pub const HALF_HOME: FieldCoordinateBounds = FieldCoordinateBounds {
        top_left: FieldCoordinate::new(0, 0),
        bottom_right: FieldCoordinate::new(12, 14),
    };
    pub const HALF_AWAY: FieldCoordinateBounds = FieldCoordinateBounds {
        top_left: FieldCoordinate::new(13, 0),
        bottom_right: FieldCoordinate::new(25, 14),
    };
    pub const LOS_HOME: FieldCoordinateBounds = FieldCoordinateBounds {
        top_left: FieldCoordinate::new(12, 4),
        bottom_right: FieldCoordinate::new(12, 10),
    };
    pub const LOS_AWAY: FieldCoordinateBounds = FieldCoordinateBounds {
        top_left: FieldCoordinate::new(13, 4),
        bottom_right: FieldCoordinate::new(13, 10),
    };
    pub const CENTER_FIELD_HOME: FieldCoordinateBounds = FieldCoordinateBounds {
        top_left: FieldCoordinate::new(0, 4),
        bottom_right: FieldCoordinate::new(11, 10),
    };
    pub const CENTER_FIELD_AWAY: FieldCoordinateBounds = FieldCoordinateBounds {
        top_left: FieldCoordinate::new(14, 4),
        bottom_right: FieldCoordinate::new(25, 10),
    };
    pub const ENDZONE_HOME: FieldCoordinateBounds = FieldCoordinateBounds {
        top_left: FieldCoordinate::new(0, 0),
        bottom_right: FieldCoordinate::new(0, 14),
    };
    pub const ENDZONE_AWAY: FieldCoordinateBounds = FieldCoordinateBounds {
        top_left: FieldCoordinate::new(25, 0),
        bottom_right: FieldCoordinate::new(25, 14),
    };
    pub const UPPER_WIDE_ZONE_HOME: FieldCoordinateBounds = FieldCoordinateBounds {
        top_left: FieldCoordinate::new(0, 0),
        bottom_right: FieldCoordinate::new(12, 3),
    };
    pub const UPPER_WIDE_ZONE_AWAY: FieldCoordinateBounds = FieldCoordinateBounds {
        top_left: FieldCoordinate::new(13, 0),
        bottom_right: FieldCoordinate::new(25, 3),
    };
    pub const LOWER_WIDE_ZONE_HOME: FieldCoordinateBounds = FieldCoordinateBounds {
        top_left: FieldCoordinate::new(0, 11),
        bottom_right: FieldCoordinate::new(12, 14),
    };
    pub const LOWER_WIDE_ZONE_AWAY: FieldCoordinateBounds = FieldCoordinateBounds {
        top_left: FieldCoordinate::new(13, 11),
        bottom_right: FieldCoordinate::new(25, 14),
    };
    pub const SIDELINE_UPPER: FieldCoordinateBounds = FieldCoordinateBounds {
        top_left: FieldCoordinate::new(1, 0),
        bottom_right: FieldCoordinate::new(24, 0),
    };
    pub const SIDELINE_LOWER: FieldCoordinateBounds = FieldCoordinateBounds {
        top_left: FieldCoordinate::new(1, 14),
        bottom_right: FieldCoordinate::new(24, 14),
    };

    pub const fn new(top_left: FieldCoordinate, bottom_right: FieldCoordinate) -> Self {
        FieldCoordinateBounds { top_left, bottom_right }
    }

    pub fn is_in_bounds(self, c: FieldCoordinate) -> bool {
        c.x >= self.top_left.x
            && c.y >= self.top_left.y
            && c.x <= self.bottom_right.x
            && c.y <= self.bottom_right.y
    }

    pub fn width(self) -> i32 {
        self.bottom_right.x - self.top_left.x + 1
    }

    pub fn height(self) -> i32 {
        self.bottom_right.y - self.top_left.y + 1
    }

    pub fn size(self) -> i32 {
        self.width() * self.height()
    }

    /// Iterate all squares in this bounds (column-major, matching Java's order).
    pub fn coordinates(self) -> Vec<FieldCoordinate> {
        let mut result = Vec::with_capacity(self.size() as usize);
        for x in self.top_left.x..=self.bottom_right.x {
            for y in self.top_left.y..=self.bottom_right.y {
                result.push(FieldCoordinate::new(x, y));
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distance_chebyshev() {
        let a = FieldCoordinate::new(0, 0);
        let b = FieldCoordinate::new(3, 1);
        assert_eq!(a.distance_in_steps(b), 3);
    }

    #[test]
    fn adjacent() {
        let a = FieldCoordinate::new(5, 5);
        assert!(a.is_adjacent(FieldCoordinate::new(6, 5)));
        assert!(a.is_adjacent(FieldCoordinate::new(6, 6)));
        assert!(!a.is_adjacent(FieldCoordinate::new(7, 5)));
    }

    #[test]
    fn transform_mirrors_field() {
        let c = FieldCoordinate::new(10, 7);
        let t = c.transform();
        assert_eq!(t.x, FIELD_WIDTH - 1 - 10);
        assert_eq!(t.y, 7);
        // Transform is its own inverse on-field
        assert_eq!(t.transform(), c);
    }

    #[test]
    fn transform_dugout() {
        let home_rsv = FieldCoordinate::new(RSV_HOME_X, 3);
        let away_rsv = home_rsv.transform();
        assert_eq!(away_rsv.x, RSV_AWAY_X);
        assert_eq!(away_rsv.transform().x, RSV_HOME_X);
    }

    #[test]
    fn direction_to() {
        let origin = FieldCoordinate::new(5, 5);
        assert_eq!(origin.direction_to(FieldCoordinate::new(6, 4)), Some(Direction::Northeast));
        assert_eq!(origin.direction_to(origin), None);
    }

    #[test]
    fn bounds_in_bounds() {
        assert!(FieldCoordinateBounds::HALF_HOME.is_in_bounds(FieldCoordinate::new(0, 0)));
        assert!(!FieldCoordinateBounds::HALF_HOME.is_in_bounds(FieldCoordinate::new(13, 0)));
    }

    #[test]
    fn bounds_size() {
        // FIELD is 26 wide × 15 tall = 390
        assert_eq!(FieldCoordinateBounds::FIELD.size(), 390);
    }

    #[test]
    fn bounds_coordinates_count() {
        let b = FieldCoordinateBounds::LOS_HOME;
        // x=12, y=4..10 → 7 squares
        assert_eq!(b.coordinates().len(), 7);
    }

    #[test]
    fn serde_round_trip() {
        let c = FieldCoordinate::new(13, 7);
        let json = serde_json::to_string(&c).unwrap();
        let back: FieldCoordinate = serde_json::from_str(&json).unwrap();
        assert_eq!(c, back);
    }

    #[test]
    fn field_width_is_26() {
        assert_eq!(FIELD_WIDTH, 26);
    }

    #[test]
    fn field_height_is_15() {
        assert_eq!(FIELD_HEIGHT, 15);
    }

    #[test]
    fn coordinate_add_returns_correct_result() {
        let c = FieldCoordinate::new(5, 5).add(2, -1);
        assert_eq!(c.x, 7);
        assert_eq!(c.y, 4);
    }

    #[test]
    fn box_coordinate_is_true_for_home_rsv() {
        assert!(FieldCoordinate::new(RSV_HOME_X, 1).is_box_coordinate());
        assert!(FieldCoordinate::new(KO_HOME_X, 1).is_box_coordinate());
        assert!(FieldCoordinate::new(RSV_AWAY_X, 1).is_box_coordinate());
    }

    #[test]
    fn pitch_coordinate_is_not_box() {
        assert!(!FieldCoordinate::new(5, 5).is_box_coordinate());
    }

    #[test]
    fn bounds_endzone_home_and_away_in_bounds() {
        assert!(FieldCoordinateBounds::ENDZONE_HOME.is_in_bounds(FieldCoordinate::new(0, 5)));
        assert!(FieldCoordinateBounds::ENDZONE_AWAY.is_in_bounds(FieldCoordinate::new(25, 5)));
    }

    #[test]
    fn is_on_pitch_returns_true_for_valid_square_and_false_for_out_of_bounds() {
        assert!(FieldCoordinate::new(0, 0).is_on_pitch());
        assert!(FieldCoordinate::new(25, 14).is_on_pitch());
        assert!(!FieldCoordinate::new(-1, 7).is_on_pitch());
        assert!(!FieldCoordinate::new(26, 0).is_on_pitch());
    }

    #[test]
    fn step_moves_by_direction_and_distance() {
        let c = FieldCoordinate::new(5, 5);
        let east_2 = c.step(Direction::East, 2);
        assert_eq!(east_2.x, 7);
        assert_eq!(east_2.y, 5);
        let south_1 = c.step(Direction::South, 1);
        assert_eq!(south_1.x, 5);
        assert_eq!(south_1.y, 6);
    }

    #[test]
    fn neighbours_returns_eight_squares() {
        let c = FieldCoordinate::new(5, 5);
        let n = c.neighbours();
        assert_eq!(n.len(), 8);
        assert!(n.iter().all(|nb| c.is_adjacent(*nb)));
    }
}
