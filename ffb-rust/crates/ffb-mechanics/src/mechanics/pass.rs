use ffb_model::enums::PassingDistance;
use ffb_model::types::FieldCoordinate;

/// BB2025 passing range table — mirrors Java `bb2025/PassMechanic.throwingRangeTable()`.
/// Indexed by [deltaY][deltaX] where deltaX/Y = abs coordinate difference.
/// Returns `None` for (0,0) (can't pass to own square) or out-of-range indices.
pub fn passing_distance_bb2025(from: FieldCoordinate, to: FieldCoordinate) -> Option<PassingDistance> {
    let dx = (from.x - to.x).unsigned_abs() as usize;
    let dy = (from.y - to.y).unsigned_abs() as usize;
    if dx >= 14 || dy >= 14 {
        return None;
    }
    // Table rows 0-13 represent deltaY; columns 0-13 represent deltaX.
    // 'T' = no pass (same square), ' ' = out of range → None.
    const TABLE: [[u8; 14]; 14] = [
        //  0    1    2    3    4    5    6    7    8    9   10   11   12   13
        [b'T',b'Q',b'Q',b'Q',b'S',b'S',b'S',b'L',b'L',b'L',b'L',b'B',b'B',b'B'], // dy=0
        [b'Q',b'Q',b'Q',b'Q',b'S',b'S',b'S',b'L',b'L',b'L',b'L',b'B',b'B',b'B'], // dy=1
        [b'Q',b'Q',b'Q',b'S',b'S',b'S',b'S',b'L',b'L',b'L',b'L',b'B',b'B',b'B'], // dy=2
        [b'Q',b'Q',b'S',b'S',b'S',b'S',b'S',b'L',b'L',b'L',b'B',b'B',b'B',b' '], // dy=3
        [b'S',b'S',b'S',b'S',b'S',b'S',b'L',b'L',b'L',b'L',b'B',b'B',b'B',b' '], // dy=4
        [b'S',b'S',b'S',b'S',b'S',b'L',b'L',b'L',b'L',b'B',b'B',b'B',b' ',b' '], // dy=5
        [b'S',b'S',b'S',b'S',b'L',b'L',b'L',b'L',b'L',b'B',b'B',b'B',b' ',b' '], // dy=6
        [b'L',b'L',b'L',b'L',b'L',b'L',b'L',b'L',b'B',b'B',b'B',b' ',b' ',b' '], // dy=7
        [b'L',b'L',b'L',b'L',b'L',b'L',b'L',b'B',b'B',b'B',b'B',b' ',b' ',b' '], // dy=8
        [b'L',b'L',b'L',b'L',b'L',b'B',b'B',b'B',b'B',b'B',b' ',b' ',b' ',b' '], // dy=9
        [b'L',b'L',b'L',b'B',b'B',b'B',b'B',b'B',b'B',b' ',b' ',b' ',b' ',b' '], // dy=10
        [b'B',b'B',b'B',b'B',b'B',b'B',b'B',b' ',b' ',b' ',b' ',b' ',b' ',b' '], // dy=11
        [b'B',b'B',b'B',b'B',b'B',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' '], // dy=12
        [b'B',b'B',b'B',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' ',b' '], // dy=13
    ];
    match TABLE[dy][dx] {
        b'Q' => Some(PassingDistance::QuickPass),
        b'S' => Some(PassingDistance::ShortPass),
        b'L' => Some(PassingDistance::LongPass),
        b'B' => Some(PassingDistance::LongBomb),
        _    => None, // 'T' (same square) or ' ' (out of range)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_square_is_none() {
        let c = FieldCoordinate::new(5, 5);
        assert_eq!(passing_distance_bb2025(c, c), None);
    }

    #[test]
    fn adjacent_square_is_quick_pass() {
        let from = FieldCoordinate::new(5, 5);
        let to = FieldCoordinate::new(6, 5);
        assert_eq!(passing_distance_bb2025(from, to), Some(PassingDistance::QuickPass));
    }

    #[test]
    fn far_square_is_long_bomb() {
        let from = FieldCoordinate::new(0, 0);
        let to = FieldCoordinate::new(11, 0);
        assert_eq!(passing_distance_bb2025(from, to), Some(PassingDistance::LongBomb));
    }

    #[test]
    fn out_of_range_is_none() {
        let from = FieldCoordinate::new(0, 0);
        let to = FieldCoordinate::new(14, 0);
        assert_eq!(passing_distance_bb2025(from, to), None);
    }

    #[test]
    fn direction_independent() {
        // Java's table is indexed by absolute delta, so swapping from/to shouldn't matter.
        let from = FieldCoordinate::new(5, 5);
        let to = FieldCoordinate::new(9, 5);
        assert_eq!(passing_distance_bb2025(from, to), passing_distance_bb2025(to, from));
    }
}
