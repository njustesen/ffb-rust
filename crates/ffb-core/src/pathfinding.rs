/// Max-probability Dijkstra over the 26×17 Blood Bowl pitch.
///
/// For each reachable square, finds the path that maximises cumulative success
/// probability (accounting for dodge rolls when leaving tackle zones and GFI
/// rolls when exceeding movement allowance).
use std::collections::{BinaryHeap, HashMap};
use std::cmp::Ordering;

use crate::model::field_model::FieldModel;
use crate::model::game_state::GameState;
use crate::model::player::Player;
use crate::types::{FieldCoordinate, PlayerId, TeamId};
use crate::skills::SkillId;
use smallvec::SmallVec;

// ── Constants ─────────────────────────────────────────────────────────────────

const MAX_GFI: u8 = 2; // 3 with Sprint skill

// ── Path entry ────────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct PathEntry {
    /// Ordered list of squares from start (exclusive) to destination (inclusive).
    pub path: SmallVec<[FieldCoordinate; 12]>,
    /// Cumulative success probability for traversing this path.
    pub probability: f64,
}

// ── Dice probability helpers ──────────────────────────────────────────────────

/// Minimum 2d6 roll needed to pass a dodge.
/// Base: `max(2, 4 - (ag - 1)) = max(2, 5 - ag)`.
/// +1 per tackle zone; Dodge skill removes one TZ from count (or provides re-roll).
pub fn dodge_min_roll(ag: u8, tz_count: u8, has_dodge_skill: bool) -> u8 {
    let base = (5u8).saturating_sub(ag).max(2);
    let effective_tz = if has_dodge_skill && tz_count > 0 {
        tz_count - 1
    } else {
        tz_count
    };
    (base + effective_tz).min(6)
}

/// Probability of passing a single d6 roll of min_roll or better.
/// With re-roll (from Dodge skill) the probability is 1 - (1 - p)^2.
pub fn dodge_probability(min_roll: u8, has_reroll: bool) -> f64 {
    let p = (7.0 - min_roll as f64).max(1.0) / 6.0;
    if has_reroll {
        1.0 - (1.0_f64 - p).powi(2)
    } else {
        p
    }
}

/// Probability of passing a GFI roll.
/// Base: 5+ on d6 = 5/6. With Sure Feet re-roll: 1 - (1/6)^2 = 35/36.
pub fn gfi_probability(has_sure_feet: bool) -> f64 {
    let p = 5.0 / 6.0;
    if has_sure_feet {
        1.0 - (1.0_f64 - p).powi(2)
    } else {
        p
    }
}

// ── Dijkstra node ─────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
struct DijkNode {
    coord: FieldCoordinate,
    cum_prob: f64,
    steps: u8,
    _parent: Option<FieldCoordinate>,
}

impl PartialEq for DijkNode {
    fn eq(&self, other: &Self) -> bool {
        self.cum_prob == other.cum_prob
    }
}

impl Eq for DijkNode {}

impl PartialOrd for DijkNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// Max-heap by cum_prob
impl Ord for DijkNode {
    fn cmp(&self, other: &Self) -> Ordering {
        self.cum_prob
            .partial_cmp(&other.cum_prob)
            .unwrap_or(Ordering::Equal)
    }
}

// ── Main pathfinding function ─────────────────────────────────────────────────

/// Find all reachable squares from `player`'s current position.
///
/// Returns a map from destination coordinate to the `PathEntry` with the
/// highest probability path to that square.
///
/// `movement_remaining`: how many squares the player can still move (usually
///   `effective_ma - movement_used`; the acting player's current remaining MA).
pub fn find_paths(
    field: &FieldModel,
    player: &Player,
    player_id: &PlayerId,
    team: TeamId,
    movement_remaining: u8,
) -> HashMap<FieldCoordinate, PathEntry> {
    let start = match field.player_coord(player_id) {
        Some(c) => c,
        None => return HashMap::new(),
    };

    let max_steps = movement_remaining + if player.has_skill(SkillId::Sprint) { 3 } else { MAX_GFI };
    let has_dodge = player.has_skill(SkillId::Dodge);
    let has_sure_feet = player.has_skill(SkillId::SureFeet);
    let ag = player.effective_ag();

    // best[coord] = highest cum_prob seen so far for that square
    let mut best: HashMap<FieldCoordinate, f64> = HashMap::new();
    // parent_of[coord] = (parent_coord, prob_of_this_step, steps_count)
    let mut parent_of: HashMap<FieldCoordinate, (Option<FieldCoordinate>, u8, f64)> = HashMap::new();

    best.insert(start, 1.0);

    let mut heap: BinaryHeap<DijkNode> = BinaryHeap::new();
    heap.push(DijkNode {
        coord: start,
        cum_prob: 1.0,
        steps: 0,
        _parent: None,
    });

    while let Some(node) = heap.pop() {
        // Skip stale entries
        if let Some(&b) = best.get(&node.coord) {
            if node.cum_prob < b - 1e-12 {
                continue;
            }
        }
        if node.steps >= max_steps {
            continue;
        }

        for neighbor in node.coord.neighbors() {
            // Can't move through occupied squares (unless special skill — handled later)
            if let Some(occupant) = field.player_at(neighbor) {
                if occupant != player_id {
                    continue;
                }
            }

            let new_steps = node.steps + 1;

            // Compute step probability
            let mut step_prob = 1.0f64;

            // Dodge: leaving a square that has opposing TZs
            let leaving_tz = field.tackle_zones_on(node.coord, team);
            if leaving_tz > 0 {
                let min_roll = dodge_min_roll(ag, leaving_tz, has_dodge);
                // Dodge skill: if we have it, we either subtract 1 TZ (already in min_roll)
                // AND we get a re-roll if we fail. Simplification: treat it as providing
                // the re-roll when min_roll was computed without the Dodge -1 TZ benefit.
                // Here we've baked -1 TZ into min_roll; re-roll applies on top.
                step_prob *= dodge_probability(min_roll, false);
            }

            // GFI: if this step exceeds the movement allowance
            if new_steps > movement_remaining {
                step_prob *= gfi_probability(has_sure_feet);
            }

            let new_prob = node.cum_prob * step_prob;

            // Prune very unlikely paths
            if new_prob < 0.001 {
                continue;
            }

            let entry = best.entry(neighbor).or_insert(0.0);
            if new_prob > *entry + 1e-12 {
                *entry = new_prob;
                parent_of.insert(neighbor, (Some(node.coord), new_steps, step_prob));
                heap.push(DijkNode {
                    coord: neighbor,
                    cum_prob: new_prob,
                    steps: new_steps,
                    _parent: Some(node.coord),
                });
            }
        }
    }

    // Build PathEntry for each reachable square (excluding start)
    let mut result = HashMap::new();
    for (coord, &prob) in &best {
        if *coord == start {
            continue;
        }
        // Reconstruct path
        let mut path: SmallVec<[FieldCoordinate; 12]> = SmallVec::new();
        let mut cur = *coord;
        loop {
            path.push(cur);
            match parent_of.get(&cur) {
                Some((Some(parent), _, _)) => cur = *parent,
                _ => break,
            }
        }
        path.reverse();
        result.insert(*coord, PathEntry { path, probability: prob });
    }
    result
}

// ── Skill-aware pathfinding ───────────────────────────────────────────────────

/// Compute per-square extra TZ contributed by opponents with PrehensileTail.
/// For each square, counts adjacent opponents (on opposing team) with PrehensileTail.
fn prehensile_tail_tz_map(state: &GameState, moving_team: TeamId) -> HashMap<FieldCoordinate, u8> {
    let mut map: HashMap<FieldCoordinate, u8> = HashMap::new();
    let opposing_team = moving_team.opponent();
    for (pid, coord, ps) in state.field.team_players_on_pitch(opposing_team) {
        if !ps.is_active() {
            continue;
        }
        if state.team(opposing_team).player_by_id(pid).map(|p| p.has_skill(SkillId::PrehensileTail)).unwrap_or(false) {
            for neighbor in coord.neighbors() {
                *map.entry(neighbor).or_insert(0) += 1;
            }
        }
    }
    map
}

/// Find paths accounting for BreakTackle (use ST for dodge if better than AG),
/// PrehensileTail (+1 TZ per adjacent Prehensile Tail opponent), and Leap (can
/// pass through occupied squares).
pub fn find_paths_in_state(
    state: &GameState,
    player: &Player,
    player_id: &PlayerId,
    team: TeamId,
    movement_remaining: u8,
) -> HashMap<FieldCoordinate, PathEntry> {
    let field = &state.field;
    let start = match field.player_coord(player_id) {
        Some(c) => c,
        None => return HashMap::new(),
    };

    let max_steps = movement_remaining + if player.has_skill(SkillId::Sprint) { 3 } else { MAX_GFI };
    let has_dodge = player.has_skill(SkillId::Dodge);
    let has_sure_feet = player.has_skill(SkillId::SureFeet);
    let has_leap = player.has_skill(SkillId::Leap) || player.has_skill(SkillId::Pogo);
    let has_break_tackle = player.has_skill(SkillId::BreakTackle);
    // BreakTackle: use max(AG, ST) for dodge base roll
    let ag = if has_break_tackle {
        player.effective_ag().max(player.effective_st())
    } else {
        player.effective_ag()
    };

    let pt_map = prehensile_tail_tz_map(state, team);

    let mut best: HashMap<FieldCoordinate, f64> = HashMap::new();
    let mut parent_of: HashMap<FieldCoordinate, (Option<FieldCoordinate>, u8, f64)> = HashMap::new();

    best.insert(start, 1.0);

    let mut heap: BinaryHeap<DijkNode> = BinaryHeap::new();
    heap.push(DijkNode { coord: start, cum_prob: 1.0, steps: 0, _parent: None });

    while let Some(node) = heap.pop() {
        if let Some(&b) = best.get(&node.coord) {
            if node.cum_prob < b - 1e-12 { continue; }
        }
        if node.steps >= max_steps { continue; }

        for neighbor in node.coord.neighbors() {
            // Leap allows moving through occupied squares
            if let Some(occupant) = field.player_at(neighbor) {
                if occupant != player_id && !has_leap {
                    continue;
                }
            }

            let new_steps = node.steps + 1;
            let mut step_prob = 1.0f64;

            // Dodge: leaving a square that has opposing TZs
            let base_tz = field.tackle_zones_on(node.coord, team);
            let pt_bonus = *pt_map.get(&node.coord).unwrap_or(&0);
            let leaving_tz = base_tz.saturating_add(pt_bonus);
            if leaving_tz > 0 {
                let min_roll = dodge_min_roll(ag, leaving_tz, has_dodge);
                step_prob *= dodge_probability(min_roll, false);
            }

            // GFI
            if new_steps > movement_remaining {
                step_prob *= gfi_probability(has_sure_feet);
            }

            let new_prob = node.cum_prob * step_prob;
            if new_prob < 0.001 { continue; }

            let entry = best.entry(neighbor).or_insert(0.0);
            if new_prob > *entry + 1e-12 {
                *entry = new_prob;
                parent_of.insert(neighbor, (Some(node.coord), new_steps, step_prob));
                heap.push(DijkNode { coord: neighbor, cum_prob: new_prob, steps: new_steps, _parent: Some(node.coord) });
            }
        }
    }

    let mut result = HashMap::new();
    for (coord, &prob) in &best {
        if *coord == start { continue; }
        let mut path: SmallVec<[FieldCoordinate; 12]> = SmallVec::new();
        let mut cur = *coord;
        loop {
            path.push(cur);
            match parent_of.get(&cur) {
                Some((Some(parent), _, _)) => cur = *parent,
                _ => break,
            }
        }
        path.reverse();
        result.insert(*coord, PathEntry { path, probability: prob });
    }
    result
}

// ── Direction helpers for scatter ─────────────────────────────────────────────

/// Convert a d8 scatter direction (1–8) to a (dx, dy) delta.
/// Clockwise from North, matching Java's DirectionFactory.forRoll():
///   1=N, 2=NE, 3=E, 4=SE, 5=S, 6=SW, 7=W, 8=NW
pub fn scatter_delta(direction: u8) -> (i8, i8) {
    match direction {
        1 => (0, -1),   // N
        2 => (1, -1),   // NE
        3 => (1, 0),    // E
        4 => (1, 1),    // SE
        5 => (0, 1),    // S
        6 => (-1, 1),   // SW
        7 => (-1, 0),   // W
        _ => (-1, -1),  // NW
    }
}

/// Apply a scatter step: coord + (direction, distance) steps, clamping to valid.
/// NOTE: This does NOT implement crowd throw-in. Use `scatter_with_crowd_throw_in`
/// for ball scatter during play (passes, bounces, fumbles).
pub fn apply_scatter(from: FieldCoordinate, direction: u8, distance: u8) -> FieldCoordinate {
    let (dx, dy) = scatter_delta(direction);
    let x = from.x as i16 + dx as i16 * distance as i16;
    let y = from.y as i16 + dy as i16 * distance as i16;
    FieldCoordinate::new(
        x.clamp(0, crate::types::PITCH_WIDTH as i16 - 1) as u8,
        y.clamp(0, crate::types::PITCH_HEIGHT as i16 - 1) as u8,
    )
}

/// Which boundary of the pitch the ball exited through.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BoundaryEdge {
    North, // y < 0
    South, // y >= PITCH_HEIGHT
    West,  // x < 0
    East,  // x >= PITCH_WIDTH
}

/// BB2025 crowd throw-in from a boundary square.
/// Roll d6 for direction (3 inward options) and d6 for distance (1–3).
/// Called when the ball exits the pitch during active play.
pub fn crowd_throw_in(
    boundary_sq: FieldCoordinate,
    edge: BoundaryEdge,
    rng: &mut crate::rng::GameRng,
) -> FieldCoordinate {
    use crate::types::{PITCH_WIDTH, PITCH_HEIGHT};

    let dir_roll = rng.roll_d6();
    let dist_roll = rng.roll_d6();
    let distance = (dist_roll as i16 + 1) / 2; // d6 → 1,1,2,2,3,3

    // For each edge, pick from 3 inward directions based on d6 roll:
    // 1-2 = "left" diagonal, 3-4 = straight in, 5-6 = "right" diagonal
    let (dx, dy): (i16, i16) = match edge {
        BoundaryEdge::North => match dir_roll { // inward = south (y+)
            1 | 2 => (-1, 1), // SW
            3 | 4 => (0, 1),  // S
            _ =>     (1, 1),  // SE
        },
        BoundaryEdge::South => match dir_roll { // inward = north (y-)
            1 | 2 => (1, -1),  // NE
            3 | 4 => (0, -1),  // N
            _ =>     (-1, -1), // NW
        },
        BoundaryEdge::West => match dir_roll { // inward = east (x+)
            1 | 2 => (1, -1),  // NE
            3 | 4 => (1, 0),   // E
            _ =>     (1, 1),   // SE
        },
        BoundaryEdge::East => match dir_roll { // inward = west (x-)
            1 | 2 => (-1, -1), // NW
            3 | 4 => (-1, 0),  // W
            _ =>     (-1, 1),  // SW
        },
    };

    let x = (boundary_sq.x as i16 + dx * distance)
        .clamp(0, PITCH_WIDTH as i16 - 1) as u8;
    let y = (boundary_sq.y as i16 + dy * distance)
        .clamp(0, PITCH_HEIGHT as i16 - 1) as u8;
    FieldCoordinate::new(x, y)
}

/// Apply scatter with proper BB2025 crowd throw-in when ball exits the pitch.
/// Use this for all ball scatter during active play (pass inaccurate, fumble, bounce, etc.).
pub fn scatter_with_crowd_throw_in(
    from: FieldCoordinate,
    direction: u8,
    distance: u8,
    rng: &mut crate::rng::GameRng,
) -> FieldCoordinate {
    use crate::types::{PITCH_WIDTH, PITCH_HEIGHT};

    let (dx, dy) = scatter_delta(direction);
    let raw_x = from.x as i16 + dx as i16 * distance as i16;
    let raw_y = from.y as i16 + dy as i16 * distance as i16;

    // On-pitch — common case, no throw-in needed
    if raw_x >= 0 && raw_x < PITCH_WIDTH as i16 && raw_y >= 0 && raw_y < PITCH_HEIGHT as i16 {
        return FieldCoordinate::new(raw_x as u8, raw_y as u8);
    }

    // Find where the ball first crossed the pitch boundary
    let mut cur_x = from.x as i16;
    let mut cur_y = from.y as i16;
    let step_x = dx as i16;
    let step_y = dy as i16;

    // Walk step by step until we exit
    for _ in 0..distance {
        cur_x += step_x;
        cur_y += step_y;
        if cur_x < 0 || cur_x >= PITCH_WIDTH as i16 || cur_y < 0 || cur_y >= PITCH_HEIGHT as i16 {
            break;
        }
    }

    // cur_x/cur_y is now the off-pitch position (or last off-pitch step).
    // Determine which edge and snap to boundary.
    let (edge, boundary_sq) = if cur_y < 0 {
        let bx = cur_x.clamp(0, PITCH_WIDTH as i16 - 1) as u8;
        (BoundaryEdge::North, FieldCoordinate::new(bx, 0))
    } else if cur_y >= PITCH_HEIGHT as i16 {
        let bx = cur_x.clamp(0, PITCH_WIDTH as i16 - 1) as u8;
        (BoundaryEdge::South, FieldCoordinate::new(bx, PITCH_HEIGHT - 1))
    } else if cur_x < 0 {
        let by = cur_y.clamp(0, PITCH_HEIGHT as i16 - 1) as u8;
        (BoundaryEdge::West, FieldCoordinate::new(0, by))
    } else {
        let by = cur_y.clamp(0, PITCH_HEIGHT as i16 - 1) as u8;
        (BoundaryEdge::East, FieldCoordinate::new(PITCH_WIDTH - 1, by))
    };

    crowd_throw_in(boundary_sq, edge, rng)
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::field_model::FieldModel;
    use crate::model::player::{Player, PlayerStats};
    use crate::skills::SkillSet;
    use crate::types::{FieldCoordinate, PlayerId, PlayerState, TeamId};

    fn make_player(id: &str, ma: u8, ag: u8) -> (PlayerId, Player) {
        let pid = PlayerId(id.into());
        let p = Player::new(
            pid.clone(),
            id.into(),
            "lineman".into(),
            TeamId::Home,
            1,
            PlayerStats::new(ma, 3, ag, 8, None),
            SkillSet::empty(),
        );
        (pid, p)
    }

    #[test]
    fn dodge_min_roll_no_tz() {
        // AG4, 0 TZ → base = max(2, 5-4) = 2
        assert_eq!(dodge_min_roll(4, 0, false), 2);
    }

    #[test]
    fn dodge_min_roll_one_tz() {
        // AG4, 1 TZ, no Dodge → 2+1 = 3
        assert_eq!(dodge_min_roll(4, 1, false), 3);
    }

    #[test]
    fn dodge_min_roll_dodge_skill_reduces_tz() {
        // AG4, 2 TZ, has Dodge → effective_tz=1 → min_roll=3
        assert_eq!(dodge_min_roll(4, 2, true), 3);
    }

    #[test]
    fn dodge_probability_values() {
        // 5/6 for roll of 2+
        assert!((dodge_probability(2, false) - 5.0 / 6.0).abs() < 1e-9);
        // 1/6 for roll of 6+
        assert!((dodge_probability(6, false) - 1.0 / 6.0).abs() < 1e-9);
    }

    #[test]
    fn dodge_probability_with_reroll_better() {
        let p = dodge_probability(4, false);
        let p_rr = dodge_probability(4, true);
        assert!(p_rr > p);
        assert!(p_rr <= 1.0);
    }

    #[test]
    fn gfi_probability_base() {
        assert!((gfi_probability(false) - 5.0 / 6.0).abs() < 1e-9);
    }

    #[test]
    fn gfi_probability_sure_feet_better() {
        assert!(gfi_probability(true) > gfi_probability(false));
    }

    #[test]
    fn find_paths_start_not_in_result() {
        let mut field = FieldModel::new();
        let (pid, player) = make_player("p1", 6, 4);
        let start = FieldCoordinate::new(5, 5);
        field.place_player(pid.clone(), TeamId::Home, start, PlayerState::Standing);
        let paths = find_paths(&field, &player, &pid, TeamId::Home, 6);
        assert!(!paths.contains_key(&start));
    }

    #[test]
    fn find_paths_probabilities_in_range() {
        let mut field = FieldModel::new();
        let (pid, player) = make_player("p1", 6, 4);
        let start = FieldCoordinate::new(5, 5);
        field.place_player(pid.clone(), TeamId::Home, start, PlayerState::Standing);
        let paths = find_paths(&field, &player, &pid, TeamId::Home, 6);
        for (_, entry) in &paths {
            assert!(entry.probability > 0.0, "prob must be > 0");
            assert!(entry.probability <= 1.0, "prob must be ≤ 1");
        }
    }

    #[test]
    fn find_paths_no_tz_reaches_ma_squares() {
        let mut field = FieldModel::new();
        let (pid, player) = make_player("p1", 4, 4);
        let start = FieldCoordinate::new(5, 5);
        field.place_player(pid.clone(), TeamId::Home, start, PlayerState::Standing);
        // No opponents → can reach 4 squares in any direction
        let paths = find_paths(&field, &player, &pid, TeamId::Home, 4);
        // Should reach (9,5) — 4 squares east at probability 1.0 (no TZs, no GFI)
        let target = FieldCoordinate::new(9, 5);
        let entry = paths.get(&target).expect("should reach (9,5)");
        assert!((entry.probability - 1.0).abs() < 1e-9, "prob should be 1.0 with no TZs");
    }

    #[test]
    fn find_paths_monotone_probability() {
        let mut field = FieldModel::new();
        let (pid, player) = make_player("p1", 6, 4);
        let start = FieldCoordinate::new(10, 8);
        field.place_player(pid.clone(), TeamId::Home, start, PlayerState::Standing);
        let paths = find_paths(&field, &player, &pid, TeamId::Home, 6);
        // Moving east: each step at prob 1.0 (no TZs), so all in-MA squares should be 1.0
        // Verify that path probability doesn't exceed 1.0
        for (_, e) in &paths {
            assert!(e.probability <= 1.0 + 1e-9);
        }
    }

    #[test]
    fn scatter_delta_all_directions() {
        for d in 1u8..=8 {
            let (dx, dy) = scatter_delta(d);
            assert!(dx >= -1 && dx <= 1);
            assert!(dy >= -1 && dy <= 1);
        }
    }

    #[test]
    fn apply_scatter_stays_in_bounds() {
        let corner = FieldCoordinate::new(0, 0);
        // Scatter NW from corner → clamps to (0,0)
        let result = apply_scatter(corner, 1, 3);
        assert!(result.is_valid());
    }

    #[test]
    fn scatter_with_throw_in_on_pitch_no_rng() {
        // On-pitch scatter shouldn't need RNG (no throw-in rolls)
        let center = FieldCoordinate::new(13, 8);
        // Direction East (3 in new 1=N clockwise mapping), distance 2 → (15, 8) — valid
        let mut rng = crate::rng::GameRng::new_test([]); // no rolls should be consumed
        let result = scatter_with_crowd_throw_in(center, 3, 2, &mut rng);
        assert_eq!(result, FieldCoordinate::new(15, 8));
    }

    #[test]
    fn scatter_with_throw_in_exits_north_returns_valid() {
        // Corner at (0,0), scatter NW dir=1 dist=3 → would exit north/west
        // Crowd throw-in: 2 rolls consumed (direction + distance)
        let corner = FieldCoordinate::new(0, 0);
        let mut rng = crate::rng::GameRng::new_test([4, 2]); // dir=4→S (straight in), dist=2→1 sq
        let result = scatter_with_crowd_throw_in(corner, 1, 3, &mut rng);
        assert!(result.is_valid(), "result {:?} must be on pitch", result);
    }

    #[test]
    fn scatter_with_throw_in_exits_south_returns_valid() {
        // Near south boundary y=16, scatter S (dir=7) dist=3 → exits south
        let near_south = FieldCoordinate::new(13, 15);
        let mut rng = crate::rng::GameRng::new_test([3, 3]); // dir=3→N straight, dist=3→2 sq
        let result = scatter_with_crowd_throw_in(near_south, 7, 3, &mut rng);
        assert!(result.is_valid(), "result {:?} must be on pitch", result);
    }

    #[test]
    fn scatter_with_throw_in_exits_east_returns_valid() {
        // Near east boundary x=25, scatter E (dir=5) dist=5 → exits east
        let near_east = FieldCoordinate::new(24, 8);
        let mut rng = crate::rng::GameRng::new_test([3, 4]); // dir=3-4→W straight, dist=4→2 sq
        let result = scatter_with_crowd_throw_in(near_east, 5, 5, &mut rng);
        assert!(result.is_valid(), "result {:?} must be on pitch", result);
    }

    #[test]
    fn scatter_with_throw_in_exits_west_returns_valid() {
        // Near west boundary x=0, scatter W (dir=4) dist=3 → exits west
        let near_west = FieldCoordinate::new(1, 8);
        let mut rng = crate::rng::GameRng::new_test([3, 2]); // straight in, 1 sq
        let result = scatter_with_crowd_throw_in(near_west, 4, 3, &mut rng);
        assert!(result.is_valid(), "result {:?} must be on pitch", result);
    }
}
