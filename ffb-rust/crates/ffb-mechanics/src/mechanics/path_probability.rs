use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};

use ffb_model::types::FieldCoordinate;
use ffb_model::enums::Rules;
use super::minimum_roll_dodge_bb2016;

/// Standard maximum GFI squares (Sprint adds 1 more via `extra_gfi`).
const MAX_GFI: i32 = 2;

static DIRS: [(i32, i32); 8] = [
    (-1, -1), (-1, 0), (-1, 1),
    ( 0, -1),           ( 0, 1),
    ( 1, -1), ( 1, 0),  ( 1, 1),
];

// ── Public input / output types ────────────────────────────────────────────

/// One opponent on the field with the skill flags the path finder needs.
#[derive(Debug, Clone)]
pub struct OpponentOnField {
    pub coord: FieldCoordinate,
    /// Player is standing or otherwise exerting tackle zones.
    pub has_tackle_zones: bool,
    /// Player has Diving Tackle (adds +2 dodge modifier when leaving their adjacent square).
    pub has_diving_tackle: bool,
    /// Player has Prehensile Tail (adds +1 dodge modifier when leaving their adjacent square).
    pub has_prehensile_tail: bool,
    /// Player has Disturbing Presence (adds +1 dodge/pass/catch modifier within 3 squares).
    pub has_disturbing_presence: bool,
    /// Player has Titchy (does not add TZ modifier for opponents moving INTO their square).
    pub is_titchy: bool,
}

/// Precomputed field state view: which squares are impassable and opponent info.
pub struct PathContext {
    /// Squares occupied by any player other than the mover (cannot be entered).
    pub occupied: HashSet<FieldCoordinate>,
    /// All opposing players currently on the field with their relevant skill flags.
    pub opponents: Vec<OpponentOnField>,
}

/// Moving player's parameters for path probability computation.
pub struct PlayerMoveContext {
    /// Player's current on-field position (not included in result).
    pub start: FieldCoordinate,
    /// Movement allowance with any stat modifiers already applied.
    pub movement_allowance: i32,
    /// Moves already spent this activation (0 if not yet activated).
    pub current_move: i32,
    /// Agility value (BB2016: raw stat; BB2020/25: direct target number, e.g. 3 for a 3+ player).
    pub agility: i32,
    /// Strength (used by BreakTackle as an alternative agility base).
    pub strength: i32,
    /// Rules edition (determines which dodge target formula to use).
    pub rules: Rules,
    /// TwoHeads skill: cap effective tackle zones at destination to 1.
    pub has_two_heads: bool,
    /// Ignore tackle zones when moving (e.g. Incorporeal).
    pub ignore_tackle_zones: bool,
    /// BreakTackle available: may use ST-based target instead of AG when it's lower.
    pub has_break_tackle: bool,
    /// Net modifier to the GFI target number (positive = harder to succeed).
    pub gfi_modifier_total: i32,
    /// Extra GFI squares beyond the standard 2 (1 for Sprint, 0 normally).
    pub extra_gfi: i32,
}

/// Best path and cumulative success probability for a single destination square.
#[derive(Debug, Clone)]
pub struct PathEntry {
    /// Steps from the player's start (exclusive) to this square (inclusive).
    pub path: Vec<FieldCoordinate>,
    /// Product of all dodge/GFI roll probabilities along the path (0.0–1.0).
    pub probability: f64,
}

// ── Internal Dijkstra structures ───────────────────────────────────────────

struct NodeData {
    coord: FieldCoordinate,
    cum_prob: f64,
    step_count: i32,
    parent_idx: Option<usize>,
}

/// Max-heap item by probability.
#[derive(PartialEq)]
struct HeapItem {
    prob: f64,
    node_idx: usize,
}
impl Eq for HeapItem {}
impl PartialOrd for HeapItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
}
impl Ord for HeapItem {
    fn cmp(&self, other: &Self) -> Ordering {
        self.prob.partial_cmp(&other.prob).unwrap_or(Ordering::Equal)
    }
}

// ── Public entry point ─────────────────────────────────────────────────────

/// Returns the maximum-probability path and cumulative probability for every
/// square reachable by `player` given the field state in `field`.
///
/// The result excludes the player's start square.  Unreachable squares (blocked
/// or beyond movement + GFI range) are absent from the map.
pub fn find_all_paths(
    player: &PlayerMoveContext,
    field: &PathContext,
) -> HashMap<FieldCoordinate, PathEntry> {
    let start = player.start;
    let max_gfi = MAX_GFI + player.extra_gfi;
    let remaining_ma = player.movement_allowance - player.current_move;
    let max_steps = (remaining_ma + max_gfi).max(0);

    if max_steps == 0 {
        return HashMap::new();
    }

    // GFI probability is constant for a given player/board state.
    let gfi_min_roll = (2 + player.gfi_modifier_total).max(2);
    let gfi_prob = prob_of_rolling_at_least(gfi_min_roll);

    let mut nodes: Vec<NodeData> = Vec::with_capacity(256);
    let mut queue: BinaryHeap<HeapItem> = BinaryHeap::new();
    let mut best_prob: HashMap<FieldCoordinate, f64> = HashMap::new();
    let mut best_node_idx: HashMap<FieldCoordinate, usize> = HashMap::new();

    // Seed: player at start with probability 1.0
    nodes.push(NodeData { coord: start, cum_prob: 1.0, step_count: 0, parent_idx: None });
    queue.push(HeapItem { prob: 1.0, node_idx: 0 });
    best_prob.insert(start, 1.0);

    while let Some(item) = queue.pop() {
        let (current_coord, current_prob, current_step) = {
            let n = &nodes[item.node_idx];
            (n.coord, n.cum_prob, n.step_count)
        };

        // Discard stale heap entries (a better path to this coord was already processed).
        if let Some(&recorded) = best_prob.get(&current_coord) {
            if current_prob < recorded - 1e-12 { continue; }
        }
        // Don't expand beyond the maximum reachable steps.
        if current_step >= max_steps { continue; }

        // A dodge is needed when leaving a square that has adjacent opponent tackle zones.
        let needs_dodge = !player.ignore_tackle_zones
            && field.opponents.iter()
                .any(|o| o.has_tackle_zones && o.coord.is_adjacent(current_coord));

        for &(dx, dy) in &DIRS {
            let to = FieldCoordinate::new(current_coord.x + dx, current_coord.y + dy);
            if !to.is_on_pitch() { continue; }
            if field.occupied.contains(&to) { continue; }

            let new_step = current_step + 1;
            let total_moves = player.current_move + new_step;
            let mut step_prob = 1.0_f64;

            if needs_dodge {
                let dodge_mod = dodge_modifier(player, field, current_coord, to);
                let min_roll = dodge_min_roll(player, dodge_mod);
                step_prob *= prob_of_rolling_at_least(min_roll);
            }

            if total_moves > player.movement_allowance {
                step_prob *= gfi_prob;
            }

            let new_cum = current_prob * step_prob;
            let prev_best = best_prob.get(&to).copied().unwrap_or(0.0);

            if new_cum > prev_best + 1e-12 {
                best_prob.insert(to, new_cum);
                let node_idx = nodes.len();
                nodes.push(NodeData {
                    coord: to,
                    cum_prob: new_cum,
                    step_count: new_step,
                    parent_idx: Some(item.node_idx),
                });
                best_node_idx.insert(to, node_idx);
                queue.push(HeapItem { prob: new_cum, node_idx });
            }
        }
    }

    // Build result map, excluding the start square.
    let mut result = HashMap::new();
    for (coord, &idx) in &best_node_idx {
        if *coord == start { continue; }
        let prob = nodes[idx].cum_prob;
        let path = reconstruct_path(&nodes, idx);
        result.insert(*coord, PathEntry { path, probability: prob });
    }
    result
}

// ── Helper functions ───────────────────────────────────────────────────────

/// Probability of rolling at least `min_roll` on a D6, clamped to [1/6, 1].
pub fn prob_of_rolling_at_least(min_roll: i32) -> f64 {
    ((7 - min_roll) as f64 / 6.0).clamp(1.0 / 6.0, 1.0)
}

/// Total dodge modifier for moving from `from` to `to`.
fn dodge_modifier(
    player: &PlayerMoveContext,
    field: &PathContext,
    from: FieldCoordinate,
    to: FieldCoordinate,
) -> i32 {
    // Tackle zones at destination (Titchy players excluded per rules).
    let tz_at_dest: i32 = field.opponents.iter()
        .filter(|o| o.has_tackle_zones && !o.is_titchy && o.coord.is_adjacent(to))
        .count() as i32;
    let effective_tz = if player.has_two_heads { tz_at_dest.min(1) } else { tz_at_dest };

    // DivingTackle: +2 if any DivingTackle opponent is adjacent to FROM and TZs exist at dest.
    let diving_tackle: i32 = if effective_tz > 0
        && field.opponents.iter()
            .any(|o| o.has_tackle_zones && o.has_diving_tackle && o.coord.is_adjacent(from))
    { 2 } else { 0 };

    // PrehensileTail: +1 per adjacent PrehensileTail opponent at FROM.
    let prehensile_tail: i32 = field.opponents.iter()
        .filter(|o| o.has_prehensile_tail && o.coord.is_adjacent(from))
        .count() as i32;

    // DisturbingPresence: +1 per DP opponent within 3 squares (Chebyshev) of destination.
    let disturbing_pres: i32 = field.opponents.iter()
        .filter(|o| o.has_disturbing_presence && o.has_tackle_zones
            && (o.coord.x - to.x).abs() <= 3
            && (o.coord.y - to.y).abs() <= 3)
        .count() as i32;

    effective_tz + diving_tackle + prehensile_tail + disturbing_pres
}

/// Minimum dodge roll for the given player and total modifier.
fn dodge_min_roll(player: &PlayerMoveContext, total_mod: i32) -> i32 {
    let ag_target = match player.rules {
        Rules::Bb2016 => minimum_roll_dodge_bb2016(player.agility, total_mod),
        _ => (player.agility + total_mod).max(2),
    };
    if player.has_break_tackle {
        let bt_target = match player.rules {
            Rules::Bb2016 => minimum_roll_dodge_bb2016(player.strength, total_mod),
            _ => ((7 - player.strength) + total_mod).max(2),
        };
        ag_target.min(bt_target)
    } else {
        ag_target
    }
}

/// Walk parent links to reconstruct the path from start (exclusive) to `end_idx` (inclusive).
fn reconstruct_path(nodes: &[NodeData], end_idx: usize) -> Vec<FieldCoordinate> {
    let mut path = Vec::new();
    let mut idx = end_idx;
    loop {
        let node = &nodes[idx];
        match node.parent_idx {
            None => break,
            Some(parent_idx) => {
                path.push(node.coord);
                idx = parent_idx;
            }
        }
    }
    path.reverse();
    path
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_player(start: FieldCoordinate, ma: i32, ag: i32, rules: Rules) -> PlayerMoveContext {
        PlayerMoveContext {
            start,
            movement_allowance: ma,
            current_move: 0,
            agility: ag,
            strength: 3,
            rules,
            has_two_heads: false,
            ignore_tackle_zones: false,
            has_break_tackle: false,
            gfi_modifier_total: 0,
            extra_gfi: 0,
        }
    }

    fn empty_field() -> PathContext {
        PathContext {
            occupied: HashSet::new(),
            opponents: Vec::new(),
        }
    }

    fn opponent(coord: FieldCoordinate, tz: bool) -> OpponentOnField {
        OpponentOnField {
            coord,
            has_tackle_zones: tz,
            has_diving_tackle: false,
            has_prehensile_tail: false,
            has_disturbing_presence: false,
            is_titchy: false,
        }
    }

    /// Build a PathContext with opponents also added to occupied (the standard setup).
    fn field_with_opponents(opps: Vec<OpponentOnField>) -> PathContext {
        let occupied = opps.iter().map(|o| o.coord).collect();
        PathContext { occupied, opponents: opps }
    }

    // ── prob_of_rolling_at_least ───────────────────────────────────────────

    #[test]
    fn prob_roll_2_plus_is_five_sixths() {
        let p = prob_of_rolling_at_least(2);
        assert!((p - 5.0 / 6.0).abs() < 1e-9, "2+ = 5/6, got {p}");
    }

    #[test]
    fn prob_roll_6_plus_is_one_sixth() {
        let p = prob_of_rolling_at_least(6);
        assert!((p - 1.0 / 6.0).abs() < 1e-9, "6+ = 1/6, got {p}");
    }

    #[test]
    fn prob_roll_clamped_min_at_one_sixth() {
        // Modifiers can push min_roll above 6; result is still 1/6.
        let p = prob_of_rolling_at_least(7);
        assert!((p - 1.0 / 6.0).abs() < 1e-9, "7+ clamped to 1/6, got {p}");
    }

    #[test]
    fn prob_roll_clamped_max_at_one() {
        // min_roll ≤ 1 means automatic success.
        let p = prob_of_rolling_at_least(1);
        assert!((p - 1.0).abs() < 1e-9, "1+ clamped to 1.0, got {p}");
    }

    // ── No-dodge, no-GFI baseline ──────────────────────────────────────────

    #[test]
    fn empty_field_all_adjacent_squares_reachable_with_prob_1() {
        let start = FieldCoordinate::new(12, 7);
        let player = empty_player(start, 6, 3, Rules::Bb2020);
        let field = empty_field();

        let paths = find_all_paths(&player, &field);

        // All 8 adjacent squares should be reachable with prob = 1.0 (no dodge needed)
        for &(dx, dy) in &DIRS {
            let dest = FieldCoordinate::new(12 + dx, 7 + dy);
            let entry = paths.get(&dest).unwrap_or_else(|| panic!("Missing {:?}", dest));
            assert!((entry.probability - 1.0).abs() < 1e-9,
                "Adjacent square {:?} should be prob=1.0 on empty field, got {}", dest, entry.probability);
            assert_eq!(entry.path, vec![dest], "One-step path should be [dest]");
        }
    }

    #[test]
    fn movement_allowance_limits_reachable_squares() {
        // MA=1, empty field.
        // Squares exactly 1 step away: prob=1.0 (free move)
        // Squares exactly 2 steps away: prob=5/6 (1 GFI)
        // Squares exactly 3 steps away: prob=(5/6)² (2 GFI, max)
        // Squares exactly 4+ steps away: NOT reachable
        let start = FieldCoordinate::new(12, 7);
        let player = empty_player(start, 1, 3, Rules::Bb2020);
        let field = empty_field();

        let paths = find_all_paths(&player, &field);

        // 1-step: free
        let one = FieldCoordinate::new(13, 7);
        assert!((paths[&one].probability - 1.0).abs() < 1e-9, "1-step prob={}", paths[&one].probability);
        // 2-steps: 1 GFI
        let two = FieldCoordinate::new(14, 7);
        assert!((paths[&two].probability - 5.0/6.0).abs() < 1e-9, "2-step prob={}", paths[&two].probability);
        // 3-steps: 2 GFI
        let three = FieldCoordinate::new(15, 7);
        let exp3 = (5.0/6.0) * (5.0/6.0);
        assert!((paths[&three].probability - exp3).abs() < 1e-9, "3-step prob={}", paths[&three].probability);
        // 4-steps: beyond max_steps — not reachable
        let four = FieldCoordinate::new(16, 7);
        assert!(!paths.contains_key(&four), "4 steps from MA=1 must not be reachable");
    }

    // ── Dodge probability ──────────────────────────────────────────────────

    #[test]
    fn single_opponent_tz_at_source_requires_dodge_bb2020() {
        // Opponent at (12,8) with TZ; occupies that square.
        // Destination (13,7): not adjacent to opponent (max(|13-12|,|7-8|)=max(1,1)=1 — IS adjacent!).
        // Actually (12,8) IS adjacent to (13,7): max(1,1)=1. So TZ at dest=1 → target=4+.
        // Let's use dest=(13,6) which is NOT adjacent to (12,8): max(|13-12|,|6-8|)=max(1,2)=2 → not adjacent.
        // Dodge from (12,7) to (13,6): needs_dodge=true (opp adj to source), TZ at dest=0, modifier=0.
        // BB2020 AG3: target=3+, prob=4/6.
        let start = FieldCoordinate::new(12, 7);
        let opp_coord = FieldCoordinate::new(12, 8);
        let dest = FieldCoordinate::new(13, 6); // not adjacent to opp

        let player = empty_player(start, 6, 3, Rules::Bb2020);
        let field = field_with_opponents(vec![opponent(opp_coord, true)]);

        let paths = find_all_paths(&player, &field);
        let entry = paths.get(&dest).expect("dest must be reachable");
        let expected = 4.0 / 6.0;
        assert!((entry.probability - expected).abs() < 1e-9,
            "Dodge with AG3 and 0 TZ modifier (BB2020): prob={}, expected {}", entry.probability, expected);
    }

    #[test]
    fn dodge_modifier_1tz_at_destination_bb2020() {
        // Unit-test the dodge_modifier helper directly.
        // Opponent at (13,8) with TZ; from=(12,7), to=(13,7).
        // (13,8) is adjacent to (13,7) → tz_at_dest=1, effective_tz=1.
        // No DivingTackle, PrehensileTail, or DP → total modifier = 1.
        let from = FieldCoordinate::new(12, 7);
        let to = FieldCoordinate::new(13, 7);
        let opp_coord = FieldCoordinate::new(13, 8);
        let player = empty_player(from, 6, 3, Rules::Bb2020);
        let field = field_with_opponents(vec![opponent(opp_coord, true)]);
        let mod_val = dodge_modifier(&player, &field, from, to);
        assert_eq!(mod_val, 1, "1 TZ at dest → modifier=1, got {}", mod_val);
        // BB2020 AG3 + modifier=1 → target=4+, prob=3/6
        let min_roll = dodge_min_roll(&player, mod_val);
        assert_eq!(min_roll, 4, "BB2020 AG3 + 1 mod → target=4, got {}", min_roll);
    }

    #[test]
    fn dodge_bb2016_formula() {
        // Unit-test dodge_min_roll for BB2016.
        // AG3, modifier=1: (7-min(3,6))-1+1 = (7-3)-1+1 = 4, clamped = max(2,4) = 4+.
        // AG4, modifier=0: (7-4)-1 = 2+.
        let from = FieldCoordinate::new(12, 7);
        let player3 = empty_player(from, 6, 3, Rules::Bb2016);
        let player4 = empty_player(from, 6, 4, Rules::Bb2016);
        assert_eq!(dodge_min_roll(&player3, 1), 4, "BB2016 AG3 +1mod → 4+");
        assert_eq!(dodge_min_roll(&player3, 0), 3, "BB2016 AG3 +0mod → 3+");
        assert_eq!(dodge_min_roll(&player4, 0), 2, "BB2016 AG4 +0mod → 2+");
        assert_eq!(dodge_min_roll(&player4, 2), 4, "BB2016 AG4 +2mod → 4+");
    }

    // ── GFI probability ────────────────────────────────────────────────────

    #[test]
    fn gfi_step_probability_is_5_over_6() {
        // MA=1, empty field. Moving 2 squares requires 1 GFI (standard: 2+ = 5/6).
        let start = FieldCoordinate::new(12, 7);
        let player = empty_player(start, 1, 3, Rules::Bb2020);
        let field = empty_field();

        let paths = find_all_paths(&player, &field);
        let two_steps = FieldCoordinate::new(14, 7);
        if let Some(entry) = paths.get(&two_steps) {
            assert!((entry.probability - 5.0 / 6.0).abs() < 1e-9,
                "1 GFI step: prob={}", entry.probability);
        }
    }

    #[test]
    fn two_gfi_steps_probability_is_25_over_36() {
        // MA=1, empty field. Moving 3 squares requires 2 GFI: (5/6)^2.
        let start = FieldCoordinate::new(12, 7);
        let player = empty_player(start, 1, 3, Rules::Bb2020);
        let field = empty_field();

        let paths = find_all_paths(&player, &field);
        let three_steps = FieldCoordinate::new(15, 7);
        if let Some(entry) = paths.get(&three_steps) {
            let expected = (5.0 / 6.0) * (5.0 / 6.0);
            assert!((entry.probability - expected).abs() < 1e-9,
                "2 GFI steps: prob={}, expected {}", entry.probability, expected);
        }
    }

    #[test]
    fn gfi_modifier_reduces_probability() {
        // gfi_modifier_total=1: min_roll = max(2, 2+1) = 3+, prob = 4/6.
        let start = FieldCoordinate::new(12, 7);
        let mut player = empty_player(start, 1, 3, Rules::Bb2020);
        player.gfi_modifier_total = 1;
        let field = empty_field();

        let paths = find_all_paths(&player, &field);
        let two_steps = FieldCoordinate::new(14, 7);
        if let Some(entry) = paths.get(&two_steps) {
            let expected = 4.0 / 6.0;
            assert!((entry.probability - expected).abs() < 1e-9,
                "GFI with +1 mod: prob={}, expected {}", entry.probability, expected);
        }
    }

    // ── Blocked squares ────────────────────────────────────────────────────

    #[test]
    fn occupied_square_is_not_in_result() {
        let start = FieldCoordinate::new(12, 7);
        let blocked = FieldCoordinate::new(13, 7);
        let player = empty_player(start, 6, 3, Rules::Bb2020);
        let mut field = empty_field();
        field.occupied.insert(blocked);

        let paths = find_all_paths(&player, &field);
        assert!(!paths.contains_key(&blocked), "Occupied square must not be reachable");
    }

    #[test]
    fn blocked_square_forces_alternate_path() {
        // Block (13,7). Player at (12,7) with MA=2 reaching (14,7) must go via (13,8) or (13,6).
        let start = FieldCoordinate::new(12, 7);
        let blocked = FieldCoordinate::new(13, 7);
        let dest = FieldCoordinate::new(14, 7);
        let player = empty_player(start, 6, 3, Rules::Bb2020);
        let mut field = empty_field();
        field.occupied.insert(blocked);

        let paths = find_all_paths(&player, &field);
        let entry = paths.get(&dest).expect("dest reachable via alternate route");
        // Path must not pass through blocked square
        assert!(!entry.path.contains(&blocked),
            "Path must not go through blocked square {:?}", blocked);
    }

    // ── TwoHeads skill ─────────────────────────────────────────────────────

    #[test]
    fn two_heads_caps_tz_modifier_at_one() {
        // Two opponents at (11,8) and (13,8): both adjacent to start (12,7) AND dest (12,8).
        // Both opponents occupy their squares (in occupied set).
        // Normal AG3 player: 2 TZs at dest → 3+2=5+, prob = 2/6.
        // TwoHeads: cap at 1 TZ → 3+1=4+, prob = 3/6.
        let start = FieldCoordinate::new(12, 7);
        let opp1 = FieldCoordinate::new(11, 8);
        let opp2 = FieldCoordinate::new(13, 8);
        let dest = FieldCoordinate::new(12, 8);

        let opps = vec![opponent(opp1, true), opponent(opp2, true)];
        let field = field_with_opponents(opps);

        let mut player_normal = empty_player(start, 6, 3, Rules::Bb2020);
        player_normal.has_two_heads = false;
        let paths_normal = find_all_paths(&player_normal, &field);
        let entry_normal = paths_normal.get(&dest).expect("dest reachable");
        assert!((entry_normal.probability - 2.0/6.0).abs() < 1e-9,
            "Normal 2 TZ → 5+: prob={}", entry_normal.probability);

        let field2 = field_with_opponents(vec![opponent(opp1, true), opponent(opp2, true)]);
        let mut player_th = empty_player(start, 6, 3, Rules::Bb2020);
        player_th.has_two_heads = true;
        let paths_th = find_all_paths(&player_th, &field2);
        let entry_th = paths_th.get(&dest).expect("dest reachable with TwoHeads");
        assert!((entry_th.probability - 3.0/6.0).abs() < 1e-9,
            "TwoHeads 1 TZ cap → 4+: prob={}", entry_th.probability);
    }

    // ── BreakTackle ────────────────────────────────────────────────────────

    #[test]
    fn break_tackle_uses_lower_of_ag_and_st() {
        // BB2020: AG3 dodge against 2 TZ at dest = target 5+, prob = 2/6.
        // ST5 BreakTackle: (7-5) + 2 = 4+, prob = 3/6.
        // BreakTackle should pick the lower target (4+).
        let start = FieldCoordinate::new(12, 7);
        let opp1 = FieldCoordinate::new(11, 8);
        let opp2 = FieldCoordinate::new(13, 8);
        let dest = FieldCoordinate::new(12, 8);

        let mut player = empty_player(start, 6, 3, Rules::Bb2020);
        player.strength = 5;
        player.has_break_tackle = true;

        let field = field_with_opponents(vec![opponent(opp1, true), opponent(opp2, true)]);

        let paths = find_all_paths(&player, &field);
        let entry = paths.get(&dest).expect("dest reachable");
        let expected = 3.0 / 6.0;
        assert!((entry.probability - expected).abs() < 1e-9,
            "BreakTackle ST5 with 2 TZ: prob={}, expected {}", entry.probability, expected);
    }

    // ── Path reconstruction ────────────────────────────────────────────────

    #[test]
    fn path_is_sequential_adjacent_squares() {
        let start = FieldCoordinate::new(12, 7);
        let player = empty_player(start, 6, 3, Rules::Bb2020);
        let field = empty_field();

        let paths = find_all_paths(&player, &field);

        // Check that every path consists of adjacent steps
        for (dest, entry) in &paths {
            let mut prev = start;
            for &step in &entry.path {
                assert!(prev.is_adjacent(step),
                    "Path to {:?}: non-adjacent step {:?}→{:?}", dest, prev, step);
                prev = step;
            }
            assert_eq!(prev, *dest, "Path to {:?} must end at destination", dest);
        }
    }

    #[test]
    fn start_square_not_in_result() {
        let start = FieldCoordinate::new(12, 7);
        let player = empty_player(start, 6, 3, Rules::Bb2020);
        let field = empty_field();

        let paths = find_all_paths(&player, &field);
        assert!(!paths.contains_key(&start), "Start square must not be in result");
    }

    // ── ignore_tackle_zones ────────────────────────────────────────────────

    #[test]
    fn ignore_tackle_zones_gives_prob_1_despite_opponents() {
        let start = FieldCoordinate::new(12, 7);
        let opp = FieldCoordinate::new(12, 8);
        let dest = FieldCoordinate::new(13, 7);

        let mut player = empty_player(start, 6, 3, Rules::Bb2020);
        player.ignore_tackle_zones = true;

        let field = PathContext {
            occupied: HashSet::new(),
            opponents: vec![opponent(opp, true)],
        };

        let paths = find_all_paths(&player, &field);
        let entry = paths.get(&dest).expect("dest reachable");
        assert!((entry.probability - 1.0).abs() < 1e-9,
            "ignore_tackle_zones: prob must be 1.0, got {}", entry.probability);
    }

    // ── Dijkstra finds highest-probability path ────────────────────────────

    #[test]
    fn dijkstra_finds_best_path_around_opponent() {
        // Layout: start=(10,7), opponent at (11,7) with TZ (blocks that square).
        // (12,7) must be reached via an alternate route: e.g. (10,7)→(11,6)→(12,7).
        // The path cannot go through (11,7).
        let start = FieldCoordinate::new(10, 7);
        let opp = FieldCoordinate::new(11, 7);
        let dest = FieldCoordinate::new(12, 7);

        let player = empty_player(start, 6, 3, Rules::Bb2020);
        let field = field_with_opponents(vec![opponent(opp, true)]);

        let paths = find_all_paths(&player, &field);
        let entry = paths.get(&dest).expect("dest reachable via alternate route");
        assert!(!entry.path.contains(&opp),
            "Path must not pass through blocked opponent square {:?}", opp);
        assert!(entry.probability > 0.0);
    }
}
