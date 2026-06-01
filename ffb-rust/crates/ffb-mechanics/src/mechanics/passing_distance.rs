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

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::PassingDistance;

    // ── Same square ──────────────────────────────────────────────────────────

    #[test]
    fn same_square_is_none() {
        assert_eq!(passing_distance_for_deltas(0, 0), None);
    }

    // ── Quick Pass ───────────────────────────────────────────────────────────

    #[test]
    fn quick_pass_adjacent_horizontal() {
        assert_eq!(passing_distance_for_deltas(1, 0), Some(PassingDistance::QuickPass));
    }

    #[test]
    fn quick_pass_adjacent_vertical() {
        assert_eq!(passing_distance_for_deltas(0, 1), Some(PassingDistance::QuickPass));
    }

    #[test]
    fn quick_pass_diagonal() {
        assert_eq!(passing_distance_for_deltas(1, 1), Some(PassingDistance::QuickPass));
    }

    #[test]
    fn quick_pass_row0_dx2_dx3() {
        assert_eq!(passing_distance_for_deltas(2, 0), Some(PassingDistance::QuickPass));
        assert_eq!(passing_distance_for_deltas(3, 0), Some(PassingDistance::QuickPass));
    }

    #[test]
    fn quick_pass_row1_dx2_3() {
        assert_eq!(passing_distance_for_deltas(2, 1), Some(PassingDistance::QuickPass));
        assert_eq!(passing_distance_for_deltas(3, 1), Some(PassingDistance::QuickPass));
    }

    #[test]
    fn quick_pass_row2_dx1_2() {
        assert_eq!(passing_distance_for_deltas(1, 2), Some(PassingDistance::QuickPass));
        assert_eq!(passing_distance_for_deltas(2, 2), Some(PassingDistance::QuickPass));
    }

    // ── Short Pass ───────────────────────────────────────────────────────────

    #[test]
    fn short_pass_row0_dx4_to_6() {
        assert_eq!(passing_distance_for_deltas(4, 0), Some(PassingDistance::ShortPass));
        assert_eq!(passing_distance_for_deltas(5, 0), Some(PassingDistance::ShortPass));
        assert_eq!(passing_distance_for_deltas(6, 0), Some(PassingDistance::ShortPass));
    }

    #[test]
    fn short_pass_dy4_dx0() {
        assert_eq!(passing_distance_for_deltas(0, 4), Some(PassingDistance::ShortPass));
    }

    // ── Long Pass ────────────────────────────────────────────────────────────

    #[test]
    fn long_pass_row0_dx7_to_10() {
        assert_eq!(passing_distance_for_deltas(7, 0), Some(PassingDistance::LongPass));
        assert_eq!(passing_distance_for_deltas(8, 0), Some(PassingDistance::LongPass));
        assert_eq!(passing_distance_for_deltas(9, 0), Some(PassingDistance::LongPass));
        assert_eq!(passing_distance_for_deltas(10, 0), Some(PassingDistance::LongPass));
    }

    #[test]
    fn long_pass_dy7_dx0() {
        assert_eq!(passing_distance_for_deltas(0, 7), Some(PassingDistance::LongPass));
    }

    // ── Long Bomb ────────────────────────────────────────────────────────────

    #[test]
    fn long_bomb_row0_dx11_to_13() {
        assert_eq!(passing_distance_for_deltas(11, 0), Some(PassingDistance::LongBomb));
        assert_eq!(passing_distance_for_deltas(12, 0), Some(PassingDistance::LongBomb));
        assert_eq!(passing_distance_for_deltas(13, 0), Some(PassingDistance::LongBomb));
    }

    #[test]
    fn long_bomb_dy11_to_13_dx0() {
        assert_eq!(passing_distance_for_deltas(0, 11), Some(PassingDistance::LongBomb));
        assert_eq!(passing_distance_for_deltas(0, 12), Some(PassingDistance::LongBomb));
        assert_eq!(passing_distance_for_deltas(0, 13), Some(PassingDistance::LongBomb));
    }

    #[test]
    fn long_bomb_dy13_dx2() {
        assert_eq!(passing_distance_for_deltas(2, 13), Some(PassingDistance::LongBomb));
    }

    // ── Out of range ─────────────────────────────────────────────────────────

    #[test]
    fn negative_delta_is_none() {
        assert_eq!(passing_distance_for_deltas(-1, 0), None);
        assert_eq!(passing_distance_for_deltas(0, -1), None);
    }

    #[test]
    fn delta_gte_14_is_none() {
        assert_eq!(passing_distance_for_deltas(14, 0), None);
        assert_eq!(passing_distance_for_deltas(0, 14), None);
    }

    #[test]
    fn out_of_range_table_cell_dy13_dx3() {
        // dy=13 row is "B B B   " — dx=3 maps to space → None
        assert_eq!(passing_distance_for_deltas(3, 13), None);
    }

    // ── for_coords ───────────────────────────────────────────────────────────

    #[test]
    fn for_coords_symmetric_quick_pass() {
        assert_eq!(passing_distance_for_coords(5, 7, 8, 7), Some(PassingDistance::QuickPass));
        assert_eq!(passing_distance_for_coords(8, 7, 5, 7), Some(PassingDistance::QuickPass));
    }

    #[test]
    fn for_coords_same_square_is_none() {
        assert_eq!(passing_distance_for_coords(5, 7, 5, 7), None);
    }

    #[test]
    fn for_coords_long_bomb_across_field() {
        // dx=13, dy=0 → LongBomb
        assert_eq!(passing_distance_for_coords(1, 7, 14, 7), Some(PassingDistance::LongBomb));
    }
}
