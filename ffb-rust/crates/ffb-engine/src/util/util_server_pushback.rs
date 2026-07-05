/// 1:1 translation of com.fumbbl.ffb.server.util.UtilServerPushback.
///
/// Public methods:
///   - findStartingSquare(startCoordinate, endCoordinate, homeChoice) -> PushbackSquare
///   - findPushbackSquares(Game, startingSquare, pushbackMode) -> Vec<PushbackSquare>
///
/// findPushbackSquares depends on SkillMechanic::isValidPushbackSquare and
/// PushbackMode. A simplified version is provided that handles the standard
/// 3-direction pushback plus crowd-push detection.
use ffb_model::types::{FieldCoordinate, FieldCoordinateBounds, PushbackSquare};
use ffb_model::enums::Direction;

pub struct UtilServerPushback;

impl UtilServerPushback {
    /// Java: UtilServerPushback.findStartingSquare(pStartCoordinate, pEndCoordinate, pHomeChoice)
    ///
    /// Determines the pushback direction from attacker (start) to defender (end).
    /// Returns None only when start == end (same square, should not happen in normal play).
    pub fn find_starting_square(
        start: FieldCoordinate,
        end: FieldCoordinate,
        home_choice: bool,
    ) -> Option<PushbackSquare> {
        let delta_x = start.x - end.x;
        let delta_y = start.y - end.y;

        let direction = if delta_y < 0 {
            if delta_x < 0 { Direction::Southeast }
            else if delta_x > 0 { Direction::Southwest }
            else { Direction::South }
        } else if delta_y > 0 {
            if delta_x < 0 { Direction::Northeast }
            else if delta_x > 0 { Direction::Northwest }
            else { Direction::North }
        } else {
            if delta_x < 0 { Direction::East }
            else if delta_x > 0 { Direction::West }
            else { return None; }
        };

        Some(PushbackSquare::new(end, direction, home_choice))
    }

    /// Java: UtilServerPushback.findPushbackSquares(Game, PushbackSquare, PushbackMode)
    ///
    /// Computes the 1-3 candidate pushback squares given a starting square (direction
    /// of attack) and the current pushback mode.
    ///
    /// This port handles the standard 3-direction case. SIDE_STEP and GRAB modes
    /// require SkillMechanic::isValidPushbackSquare (TODO: currently returns all empty
    /// adjacent squares as valid).
    pub fn find_pushback_squares_standard(
        starting_square: PushbackSquare,
        occupied: &dyn Fn(FieldCoordinate) -> bool,
        home_choice: bool,
    ) -> Vec<PushbackSquare> {
        let coord = starting_square.coordinate;
        let dir = starting_square.direction;

        // Compute the three candidate coordinates depending on the direction of attack.
        let nw = FieldCoordinate::new(coord.x - 1, coord.y - 1);
        let n  = FieldCoordinate::new(coord.x,     coord.y - 1);
        let ne = FieldCoordinate::new(coord.x + 1, coord.y - 1);
        let e  = FieldCoordinate::new(coord.x + 1, coord.y);
        let se = FieldCoordinate::new(coord.x + 1, coord.y + 1);
        let s  = FieldCoordinate::new(coord.x,     coord.y + 1);
        let sw = FieldCoordinate::new(coord.x - 1, coord.y + 1);
        let w  = FieldCoordinate::new(coord.x - 1, coord.y);

        let candidates: [(FieldCoordinate, Direction); 3] = match dir {
            Direction::North     => [(nw, Direction::Northwest), (n,  Direction::North),     (ne, Direction::Northeast)],
            Direction::Northeast => [(n,  Direction::North),     (ne, Direction::Northeast), (e,  Direction::East)],
            Direction::East      => [(ne, Direction::Northeast), (e,  Direction::East),       (se, Direction::Southeast)],
            Direction::Southeast => [(e,  Direction::East),       (se, Direction::Southeast), (s,  Direction::South)],
            Direction::South     => [(se, Direction::Southeast), (s,  Direction::South),     (sw, Direction::Southwest)],
            Direction::Southwest => [(s,  Direction::South),     (sw, Direction::Southwest), (w,  Direction::West)],
            Direction::West      => [(sw, Direction::Southwest), (w,  Direction::West),      (nw, Direction::Northwest)],
            Direction::Northwest => [(w,  Direction::West),      (nw, Direction::Northwest), (n,  Direction::North)],
        };

        // Filter to in-bounds squares only.
        let in_bounds: Vec<(FieldCoordinate, Direction)> = candidates
            .into_iter()
            .filter(|(c, _)| FieldCoordinateBounds::FIELD.is_in_bounds(*c))
            .collect();

        if in_bounds.is_empty() {
            return vec![];
        }

        // Prefer free squares if any exist.
        let free: Vec<(FieldCoordinate, Direction)> = in_bounds
            .iter()
            .filter(|(c, _)| !occupied(*c))
            .copied()
            .collect();

        let chosen = if !free.is_empty() { free } else {
            // Crowd-push: only valid when all three candidates are in bounds.
            if in_bounds.len() < 3 {
                return vec![];
            }
            in_bounds
        };

        chosen.into_iter()
            .map(|(c, d)| PushbackSquare::new(c, d, home_choice))
            .collect()
    }

    /// Java: `UtilServerPushback.findPushbackSquares(GRAB or SIDE_STEP mode)`.
    ///
    /// Returns all adjacent squares that are on the pitch, not occupied, and not
    /// a multi-block target coordinate (isValidPushbackSquare). Used when the
    /// attacker has `canPushBackToAnySquare` (Grab) or defender has SideStep.
    pub fn find_pushback_squares_grab(
        starting_square: PushbackSquare,
        occupied: &dyn Fn(FieldCoordinate) -> bool,
        is_valid: &dyn Fn(FieldCoordinate) -> bool,
        home_choice: bool,
    ) -> Vec<PushbackSquare> {
        let coord = starting_square.coordinate;
        let dirs = [
            (Direction::Northwest, -1i32, -1i32),
            (Direction::North,      0,   -1),
            (Direction::Northeast,  1,   -1),
            (Direction::East,       1,    0),
            (Direction::Southeast,  1,    1),
            (Direction::South,      0,    1),
            (Direction::Southwest, -1,    1),
            (Direction::West,      -1,    0),
        ];
        dirs.iter()
            .filter_map(|(dir, dx, dy)| {
                let c = FieldCoordinate::new(coord.x + dx, coord.y + dy);
                if FieldCoordinateBounds::FIELD.is_in_bounds(c) && !occupied(c) && is_valid(c) {
                    Some(PushbackSquare::new(c, *dir, home_choice))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Returns all in-bounds candidate pushback squares without filtering by occupancy.
    ///
    /// Java: `UtilServerPushback.findPushbackSquares` returns all 3 candidates (including
    /// occupied ones) — used by `StepBlockDodge.findDodgeChoice` to detect chain-push /
    /// sideline-push risks before the actual pushback happens.
    pub fn find_pushback_squares_candidates(
        starting_square: PushbackSquare,
        home_choice: bool,
    ) -> Vec<PushbackSquare> {
        let coord = starting_square.coordinate;
        let dir = starting_square.direction;

        let nw = FieldCoordinate::new(coord.x - 1, coord.y - 1);
        let n  = FieldCoordinate::new(coord.x,     coord.y - 1);
        let ne = FieldCoordinate::new(coord.x + 1, coord.y - 1);
        let e  = FieldCoordinate::new(coord.x + 1, coord.y);
        let se = FieldCoordinate::new(coord.x + 1, coord.y + 1);
        let s  = FieldCoordinate::new(coord.x,     coord.y + 1);
        let sw = FieldCoordinate::new(coord.x - 1, coord.y + 1);
        let w  = FieldCoordinate::new(coord.x - 1, coord.y);

        let candidates: [(FieldCoordinate, Direction); 3] = match dir {
            Direction::North     => [(nw, Direction::Northwest), (n,  Direction::North),     (ne, Direction::Northeast)],
            Direction::Northeast => [(n,  Direction::North),     (ne, Direction::Northeast), (e,  Direction::East)],
            Direction::East      => [(ne, Direction::Northeast), (e,  Direction::East),       (se, Direction::Southeast)],
            Direction::Southeast => [(e,  Direction::East),       (se, Direction::Southeast), (s,  Direction::South)],
            Direction::South     => [(se, Direction::Southeast), (s,  Direction::South),     (sw, Direction::Southwest)],
            Direction::Southwest => [(s,  Direction::South),     (sw, Direction::Southwest), (w,  Direction::West)],
            Direction::West      => [(sw, Direction::Southwest), (w,  Direction::West),      (nw, Direction::Northwest)],
            Direction::Northwest => [(w,  Direction::West),      (nw, Direction::Northwest), (n,  Direction::North)],
        };

        candidates
            .into_iter()
            .filter(|(c, _)| FieldCoordinateBounds::FIELD.is_in_bounds(*c))
            .map(|(c, d)| PushbackSquare::new(c, d, home_choice))
            .collect()
    }
}

impl Default for UtilServerPushback {
    fn default() -> Self { Self }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::types::{FieldCoordinate, PushbackSquare};
    use ffb_model::enums::Direction;

    fn no_occupied(_c: FieldCoordinate) -> bool { false }

    // -- find_starting_square --

    #[test]
    fn find_starting_square_attacker_north_of_defender() {
        // Attacker at (5,5), defender at (5,6) -> delta_y = 5-6 = -1 -> South direction
        let start = FieldCoordinate::new(5, 5);
        let end   = FieldCoordinate::new(5, 6);
        let sq = UtilServerPushback::find_starting_square(start, end, true).unwrap();
        assert_eq!(sq.direction, Direction::South);
        assert_eq!(sq.coordinate, end);
        assert_eq!(sq.home_choice, true);
    }

    #[test]
    fn find_starting_square_attacker_south_of_defender() {
        let start = FieldCoordinate::new(5, 7);
        let end   = FieldCoordinate::new(5, 6);
        let sq = UtilServerPushback::find_starting_square(start, end, false).unwrap();
        assert_eq!(sq.direction, Direction::North);
    }

    #[test]
    fn find_starting_square_attacker_east_of_defender() {
        let start = FieldCoordinate::new(7, 7);
        let end   = FieldCoordinate::new(6, 7);
        let sq = UtilServerPushback::find_starting_square(start, end, false).unwrap();
        assert_eq!(sq.direction, Direction::West);
    }

    #[test]
    fn find_starting_square_attacker_west_of_defender() {
        let start = FieldCoordinate::new(5, 7);
        let end   = FieldCoordinate::new(6, 7);
        let sq = UtilServerPushback::find_starting_square(start, end, true).unwrap();
        assert_eq!(sq.direction, Direction::East);
    }

    #[test]
    fn find_starting_square_diagonal_northeast() {
        // delta_x = 5-6 = -1 (< 0), delta_y = 7-6 = 1 (> 0) -> Northeast
        let start = FieldCoordinate::new(5, 7);
        let end   = FieldCoordinate::new(6, 6);
        let sq = UtilServerPushback::find_starting_square(start, end, true).unwrap();
        assert_eq!(sq.direction, Direction::Northeast);
    }

    #[test]
    fn find_starting_square_same_square_returns_none() {
        let c = FieldCoordinate::new(5, 7);
        assert!(UtilServerPushback::find_starting_square(c, c, true).is_none());
    }

    // -- find_pushback_squares_standard --

    #[test]
    fn pushback_north_produces_three_squares() {
        let start = FieldCoordinate::new(10, 7);
        let sq = PushbackSquare::new(start, Direction::North, true);
        let result = UtilServerPushback::find_pushback_squares_standard(sq, &no_occupied, true);
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn pushback_east_produces_three_squares() {
        let start = FieldCoordinate::new(10, 7);
        let sq = PushbackSquare::new(start, Direction::East, true);
        let result = UtilServerPushback::find_pushback_squares_standard(sq, &no_occupied, true);
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn pushback_on_sideline_filters_out_of_bounds() {
        // Player on y=0 pushed north -- NW and N are out of bounds (y=-1).
        let start = FieldCoordinate::new(10, 0);
        let sq = PushbackSquare::new(start, Direction::North, true);
        let result = UtilServerPushback::find_pushback_squares_standard(sq, &no_occupied, true);
        // NW=(9,-1), N=(10,-1), NE=(11,-1) -- all out of bounds -> crowd push not possible (<3).
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn pushback_prefers_free_squares_over_occupied() {
        let start = FieldCoordinate::new(10, 7);
        let sq = PushbackSquare::new(start, Direction::East, true);
        // Mark NE and SE as occupied; only E is free.
        let occupied = |c: FieldCoordinate| c.y != 7;
        let result = UtilServerPushback::find_pushback_squares_standard(sq, &occupied, true);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].direction, Direction::East);
    }

    #[test]
    fn pushback_all_occupied_returns_all_three_for_crowd_push() {
        let start = FieldCoordinate::new(10, 7);
        let sq = PushbackSquare::new(start, Direction::East, true);
        // All three squares occupied.
        let occupied = |_: FieldCoordinate| true;
        let result = UtilServerPushback::find_pushback_squares_standard(sq, &occupied, true);
        // All 3 in bounds, all occupied -> crowd push -> return all 3.
        assert_eq!(result.len(), 3);
    }
}
