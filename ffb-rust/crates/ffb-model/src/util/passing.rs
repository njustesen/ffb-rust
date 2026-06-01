use crate::types::FieldCoordinate;
use crate::enums::PassingDistance;

/// Width of the pass ruler in field units (matches Java's RULER_WIDTH).
pub const RULER_WIDTH: f64 = 1.74;

/// Returns true if a player at `interceptor` can intercept a pass from `thrower` to `target`.
///
/// Replicates the geometric ruler check from UtilPassing.canIntercept().
pub fn can_intercept(
    thrower: FieldCoordinate,
    target: FieldCoordinate,
    interceptor: FieldCoordinate,
) -> bool {
    let rx = (target.x - thrower.x) as f64;
    let ry = (target.y - thrower.y) as f64;
    let ix = (interceptor.x - thrower.x) as f64;
    let iy = (interceptor.y - thrower.y) as f64;

    let a = (rx - ix).powi(2) + (ry - iy).powi(2);
    let b = ix.powi(2) + iy.powi(2);
    let c = rx.powi(2) + ry.powi(2);

    let d1 = (ry * (ix + 0.5) - rx * (iy + 0.5)).abs();
    let d2 = (ry * (ix + 0.5) - rx * (iy - 0.5)).abs();
    let d3 = (ry * (ix - 0.5) - rx * (iy + 0.5)).abs();
    let d4 = (ry * (ix - 0.5) - rx * (iy - 0.5)).abs();

    let d_min = d1.min(d2).min(d3).min(d4);
    let sqrt_c = c.sqrt();

    c > a && c > b && RULER_WIDTH > (2.0 * d_min / sqrt_c)
}

/// Passing distance lookup table indexed by [delta_y][delta_x], matching Java's BB2020 PassMechanic.
///
/// Q=QuickPass, S=ShortPass, L=LongPass, B=LongBomb, None=invalid/out-of-range.
const TABLE: [[Option<PassingDistance>; 14]; 14] = {
    use PassingDistance::{QuickPass as Q, ShortPass as S, LongPass as L, LongBomb as B};
    // Row strings from BB2020 PassMechanic.throwingRangeTable()
    // Indexed: [dy][dx], both 0..=13
    [
        // dy=0: T Q Q Q S S S L L L L B B B
        [None,    Some(Q), Some(Q), Some(Q), Some(S), Some(S), Some(S), Some(L), Some(L), Some(L), Some(L), Some(B), Some(B), Some(B)],
        // dy=1: Q Q Q Q S S S L L L L B B B
        [Some(Q), Some(Q), Some(Q), Some(Q), Some(S), Some(S), Some(S), Some(L), Some(L), Some(L), Some(L), Some(B), Some(B), Some(B)],
        // dy=2: Q Q Q S S S S L L L L B B B
        [Some(Q), Some(Q), Some(Q), Some(S), Some(S), Some(S), Some(S), Some(L), Some(L), Some(L), Some(L), Some(B), Some(B), Some(B)],
        // dy=3: Q Q S S S S S L L L B B B (space)
        [Some(Q), Some(Q), Some(S), Some(S), Some(S), Some(S), Some(S), Some(L), Some(L), Some(L), Some(B), Some(B), Some(B), None   ],
        // dy=4: S S S S S S L L L L B B B (space)
        [Some(S), Some(S), Some(S), Some(S), Some(S), Some(S), Some(L), Some(L), Some(L), Some(L), Some(B), Some(B), Some(B), None   ],
        // dy=5: S S S S S L L L L B B B (spaces)
        [Some(S), Some(S), Some(S), Some(S), Some(S), Some(L), Some(L), Some(L), Some(L), Some(B), Some(B), Some(B), None,    None   ],
        // dy=6: S S S S L L L L L B B B (spaces)
        [Some(S), Some(S), Some(S), Some(S), Some(L), Some(L), Some(L), Some(L), Some(L), Some(B), Some(B), Some(B), None,    None   ],
        // dy=7: L L L L L L L L B B B (spaces)
        [Some(L), Some(L), Some(L), Some(L), Some(L), Some(L), Some(L), Some(L), Some(B), Some(B), Some(B), None,    None,    None   ],
        // dy=8: L L L L L L L B B B B (spaces)
        [Some(L), Some(L), Some(L), Some(L), Some(L), Some(L), Some(L), Some(B), Some(B), Some(B), Some(B), None,    None,    None   ],
        // dy=9: L L L L L B B B B B (spaces)
        [Some(L), Some(L), Some(L), Some(L), Some(L), Some(B), Some(B), Some(B), Some(B), Some(B), None,    None,    None,    None   ],
        // dy=10: L L L B B B B B B (spaces)
        [Some(L), Some(L), Some(L), Some(B), Some(B), Some(B), Some(B), Some(B), Some(B), None,    None,    None,    None,    None   ],
        // dy=11: B B B B B B B (spaces)
        [Some(B), Some(B), Some(B), Some(B), Some(B), Some(B), Some(B), None,    None,    None,    None,    None,    None,    None   ],
        // dy=12: B B B B B (spaces)
        [Some(B), Some(B), Some(B), Some(B), Some(B), None,    None,    None,    None,    None,    None,    None,    None,    None   ],
        // dy=13: B B B (spaces)
        [Some(B), Some(B), Some(B), None,    None,    None,    None,    None,    None,    None,    None,    None,    None,    None   ],
    ]
};

/// Passing distance for a throw with the given absolute coordinate deltas.
/// Returns `None` when the delta is 0,0 (same square) or exceeds the maximum range.
pub fn passing_distance_for_deltas(delta_x: i32, delta_y: i32) -> Option<PassingDistance> {
    if delta_x < 0 || delta_y < 0 || delta_x >= 14 || delta_y >= 14 {
        return None;
    }
    TABLE[delta_y as usize][delta_x as usize]
}

/// Passing distance category for a throw from `from` to `to`.
/// Returns `None` when the throw is to the same square or out of range.
pub fn passing_distance(from: FieldCoordinate, to: FieldCoordinate) -> Option<PassingDistance> {
    let dx = (to.x - from.x).abs();
    let dy = (to.y - from.y).abs();
    passing_distance_for_deltas(dx, dy)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::FieldCoordinate;
    use crate::enums::PassingDistance;

    fn c(x: i32, y: i32) -> FieldCoordinate {
        FieldCoordinate::new(x, y)
    }

    #[test]
    fn can_intercept_direct_path() {
        let thrower = c(0, 7);
        let target = c(10, 7);
        let interceptor = c(5, 7);
        assert!(can_intercept(thrower, target, interceptor));
    }

    #[test]
    fn cannot_intercept_far_off_path() {
        let thrower = c(0, 7);
        let target = c(10, 7);
        let interceptor = c(5, 12);
        assert!(!can_intercept(thrower, target, interceptor));
    }

    // ── passing_distance_for_deltas ───────────────────────────────────────────

    #[test]
    fn same_square_returns_none() {
        assert_eq!(passing_distance_for_deltas(0, 0), None);
    }

    #[test]
    fn quick_pass_horizontal() {
        // dx=1..=3, dy=0 → QuickPass (from table row dy=0: Q Q Q)
        assert_eq!(passing_distance_for_deltas(1, 0), Some(PassingDistance::QuickPass));
        assert_eq!(passing_distance_for_deltas(2, 0), Some(PassingDistance::QuickPass));
        assert_eq!(passing_distance_for_deltas(3, 0), Some(PassingDistance::QuickPass));
    }

    #[test]
    fn short_pass_horizontal() {
        // dx=4..=6, dy=0 → ShortPass
        assert_eq!(passing_distance_for_deltas(4, 0), Some(PassingDistance::ShortPass));
        assert_eq!(passing_distance_for_deltas(6, 0), Some(PassingDistance::ShortPass));
    }

    #[test]
    fn long_pass_horizontal() {
        // dx=7..=10, dy=0 → LongPass
        assert_eq!(passing_distance_for_deltas(7, 0), Some(PassingDistance::LongPass));
        assert_eq!(passing_distance_for_deltas(10, 0), Some(PassingDistance::LongPass));
    }

    #[test]
    fn long_bomb_horizontal() {
        // dx=11..=13, dy=0 → LongBomb
        assert_eq!(passing_distance_for_deltas(11, 0), Some(PassingDistance::LongBomb));
        assert_eq!(passing_distance_for_deltas(13, 0), Some(PassingDistance::LongBomb));
    }

    #[test]
    fn vertical_short_pass_dy4() {
        // dy=4, dx=0 → ShortPass (not LongPass)
        assert_eq!(passing_distance_for_deltas(0, 4), Some(PassingDistance::ShortPass));
    }

    #[test]
    fn out_of_range_returns_none() {
        assert_eq!(passing_distance_for_deltas(14, 0), None);
        assert_eq!(passing_distance_for_deltas(0, 14), None);
        assert_eq!(passing_distance_for_deltas(-1, 0), None);
    }

    #[test]
    fn table_cell_null_returns_none() {
        // dy=13, dx=3 → out-of-range cell (space in table) → None
        assert_eq!(passing_distance_for_deltas(3, 13), None);
    }

    // ── passing_distance ──────────────────────────────────────────────────────

    #[test]
    fn passing_distance_symmetrical() {
        let from = c(5, 7);
        let to = c(8, 7);
        assert_eq!(passing_distance(from, to), passing_distance(to, from));
    }

    #[test]
    fn passing_distance_long_bomb_across_field() {
        // dx=13, dy=0 → LongBomb
        assert_eq!(passing_distance(c(1, 7), c(14, 7)), Some(PassingDistance::LongBomb));
    }
}
