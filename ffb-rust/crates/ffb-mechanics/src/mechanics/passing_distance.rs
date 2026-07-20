use ffb_model::enums::PassingDistance;

/// Passing distance lookup table, indexed by [dy][dx] (both absolute deltas, 0..13).
///
/// Mirrors Java `PassingDistanceCalc` / BB2020 `PassMechanic.throwingRangeTable()`.
/// null cells (same square or out-of-range columns) are represented as `None`.
///
/// Row encoding: Q=QuickPass, S=ShortPass, L=LongPass, B=LongBomb, T/space=None.
static TABLE: [[Option<PassingDistance>; 14]; 14] = build_table();

const fn cell(c: u8) -> Option<PassingDistance> {
    match c {
        b'Q' => Some(PassingDistance::QuickPass),
        b'S' => Some(PassingDistance::ShortPass),
        b'L' => Some(PassingDistance::LongPass),
        b'B' => Some(PassingDistance::LongBomb),
        _    => None,
    }
}

/// Build the 14×14 table at compile time.
///
/// Each row string (from Java source) is indexed by dx (every 2nd character, 0-based).
const fn build_table() -> [[Option<PassingDistance>; 14]; 14] {
    // Row strings — space-separated characters; T or space = None
    // row[dy] = row string where column dx maps to char at index dx*2
    let rows: [&[u8]; 14] = [
        b"T Q Q Q S S S L L L L B B B",
        b"Q Q Q Q S S S L L L L B B B",
        b"Q Q Q S S S S L L L L B B B",
        b"Q Q S S S S S L L L B B B  ",
        b"S S S S S S L L L L B B B  ",
        b"S S S S S L L L L B B B    ",
        b"S S S S L L L L L B B B    ",
        b"L L L L L L L L B B B      ",
        b"L L L L L L L B B B B      ",
        b"L L L L L B B B B B        ",
        b"L L L B B B B B B          ",
        b"B B B B B B B              ",
        b"B B B B B                  ",
        b"B B B                      ",
    ];

    let mut table = [[None; 14]; 14];
    let mut dy = 0usize;
    while dy < 14 {
        let row = rows[dy];
        let mut dx = 0usize;
        while dx < 14 {
            let idx = dx * 2;
            let c = if idx < row.len() { row[idx] } else { b' ' };
            table[dy][dx] = cell(c);
            dx += 1;
        }
        dy += 1;
    }
    table
}

/// Returns the passing distance for the given absolute coordinate deltas.
///
/// Returns `None` for same square (0,0), out-of-range deltas (≥ 14 or < 0),
/// and cells that are off the table edge.
pub fn passing_distance_for_deltas(delta_x: i32, delta_y: i32) -> Option<PassingDistance> {
    if delta_x < 0 || delta_y < 0 || delta_x >= 14 || delta_y >= 14 {
        return None;
    }
    TABLE[delta_y as usize][delta_x as usize]
}

/// Returns the passing distance for a throw from one coordinate to another.
///
/// Uses absolute differences; returns `None` if out of range or same square.
pub fn passing_distance_for_coords(from_x: i32, from_y: i32, to_x: i32, to_y: i32) -> Option<PassingDistance> {
    passing_distance_for_deltas((to_x - from_x).abs(), (to_y - from_y).abs())
}

// Tests exercise the ffb-mechanics API directly (this module owns its own table
// transcription, separate from ffb-engine's util::passing_distance_calc mirror).
// Names and case values are aligned 1:1 with Java PassingDistanceCalcTest.
#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::PassingDistance;

    // ── Same square ──────────────────────────────────────────────────────────

    #[test]
    fn same_square_returns_null() {
        assert_eq!(passing_distance_for_deltas(0, 0), None);
    }

    // ── Quick Pass ───────────────────────────────────────────────────────────

    #[test]
    fn quick_pass() {
        let rows = [
            (1, 0), // adjacent horizontally
            (0, 1), // adjacent vertically
            (1, 1), // diagonal
            (2, 0), // two squares horizontal
            (2, 1), // row 1
            (2, 2), // row 2
            (1, 2), // row 2
            (3, 0), // row 0
            (3, 1), // row 1
        ];
        for (dx, dy) in rows {
            assert_eq!(
                passing_distance_for_deltas(dx, dy),
                Some(PassingDistance::QuickPass),
                "dx={dx}, dy={dy}"
            );
        }
    }

    // ── Short Pass ───────────────────────────────────────────────────────────

    #[test]
    fn short_pass() {
        let rows = [
            (4, 0), // row 0
            (5, 0), // row 0
            (6, 0), // row 0
            (3, 2), // row 2
            (4, 2), // row 2
            (0, 4), // dy=4
            (1, 4), // row 4
        ];
        for (dx, dy) in rows {
            assert_eq!(
                passing_distance_for_deltas(dx, dy),
                Some(PassingDistance::ShortPass),
                "dx={dx}, dy={dy}"
            );
        }
    }

    // ── Long Pass ────────────────────────────────────────────────────────────

    #[test]
    fn long_pass() {
        let rows = [
            (7, 0),  // row 0
            (8, 0),  // row 0
            (9, 0),  // row 0
            (10, 0), // row 0
            (0, 7),  // dy=7
            (1, 7),  // row 7
        ];
        for (dx, dy) in rows {
            assert_eq!(
                passing_distance_for_deltas(dx, dy),
                Some(PassingDistance::LongPass),
                "dx={dx}, dy={dy}"
            );
        }
    }

    // ── Long Bomb ────────────────────────────────────────────────────────────

    #[test]
    fn long_bomb() {
        let rows = [
            (11, 0), // row 0
            (12, 0), // row 0
            (13, 0), // row 0
            (0, 11), // dy=11
            (1, 11), // row 11
            (0, 12), // dy=12
            (0, 13), // dy=13
            (1, 12), // row 12
            (2, 13), // row 13
        ];
        for (dx, dy) in rows {
            assert_eq!(
                passing_distance_for_deltas(dx, dy),
                Some(PassingDistance::LongBomb),
                "dx={dx}, dy={dy}"
            );
        }
    }

    // ── Out of range ─────────────────────────────────────────────────────────

    #[test]
    fn negative_delta_returns_null() {
        assert_eq!(passing_distance_for_deltas(-1, 0), None);
        assert_eq!(passing_distance_for_deltas(0, -1), None);
    }

    #[test]
    fn delta_greater_than_13_returns_null() {
        assert_eq!(passing_distance_for_deltas(14, 0), None);
        assert_eq!(passing_distance_for_deltas(0, 14), None);
    }

    // ── Null cells in table (spaces) ─────────────────────────────────────────

    #[test]
    fn out_of_range_cells_return_null() {
        // dy=13, dx=3 → "B B B   " → index 3 is space → null
        assert_eq!(passing_distance_for_deltas(3, 13), None);
        // dy=13, dx=13 → bottom-right corner is off the table edge → null
        assert_eq!(passing_distance_for_deltas(13, 13), None);
    }

    // ── for_coords ───────────────────────────────────────────────────────────

    #[test]
    fn for_coordinates_symmetrical() {
        // Passing from (5,7) to (8,7) = dx=3, dy=0 → QuickPass
        assert_eq!(passing_distance_for_coords(5, 7, 8, 7), Some(PassingDistance::QuickPass));
        // Symmetric: (8,7) to (5,7)
        assert_eq!(passing_distance_for_coords(8, 7, 5, 7), Some(PassingDistance::QuickPass));
    }

    #[test]
    fn for_coordinates_same_square_returns_null() {
        assert_eq!(passing_distance_for_coords(5, 7, 5, 7), None);
    }

    #[test]
    fn for_coordinates_long_bomb_across_field() {
        // From x=1 to x=14 = dx=13, dy=0 → LongBomb
        assert_eq!(passing_distance_for_coords(1, 7, 14, 7), Some(PassingDistance::LongBomb));
    }
}
