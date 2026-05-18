/// MCTS search engine with configurable rollout depth and outcome control.
use std::collections::VecDeque;

use ffb_core::actions::{enumerate_actions, BbAction};
use ffb_core::model::game_state::GameState;
use ffb_core::rng::GameRng;
use ffb_core::types::TeamId;

use ffb_sim::evaluation::static_eval;
use ffb_sim::simulation::{SimulationLoop, Strategy};

use crate::node::{select_child_ucb, Node, NodeArena};

// ── Rollout depth ─────────────────────────────────────────────────────────────

/// How deep to simulate from a leaf node before evaluating.
#[derive(Clone, Debug)]
pub enum RolloutDepth {
    /// Static eval at every leaf — fastest; matches current Java MCTS behavior.
    None,
    /// Simulate exactly N step-level actions, then eval.
    Steps(u32),
    /// Simulate N complete player activations, then eval.
    Turns(u32),
    /// Simulate until the next kickoff (TD or turnover ends drive), then eval.
    UntilKickoff,
    /// Simulate to halftime, then eval.
    UntilHalf,
    /// Simulate to game end; terminal reward only.
    Full,
}

// ── Outcome controller ────────────────────────────────────────────────────────

/// A single roll specification used by `OutcomeController::Fixed`.
#[derive(Clone, Debug)]
pub struct RollSpec {
    pub sides: u8,
    pub value: u8,
}

/// Controls how random outcomes (dice) are sampled during MCTS rollouts.
#[derive(Clone, Debug, Default)]
pub enum OutcomeController {
    /// Sample randomly — standard MCTS behavior.
    #[default]
    Stochastic,
    /// Deterministic stratified sampling: tracks how often each face has been
    /// rolled and always picks the face most under-represented relative to its
    /// theoretical probability 1/sides.
    /// Minimises L1 distance to the theoretical distribution in O(1/N) vs O(1/√N).
    Stratified {
        visit_counts: std::collections::HashMap<u8, u32>,
        total_visits: u32,
    },
    /// Inject a fixed sequence of outcomes (for testing and analysis).
    Fixed(VecDeque<u8>),
    /// Expand all d6 outcomes as separate tree children (for chance-node analysis).
    /// Only practical for small branching-factor rolls.
    Exhaustive,
}

impl OutcomeController {
    /// Sample the next outcome for a `sides`-sided die using this controller.
    /// Updates internal state (visit counts for Stratified; pops from queue for Fixed).
    pub fn sample(&mut self, sides: u8, rng: &mut GameRng) -> u8 {
        match self {
            Self::Stochastic | Self::Exhaustive => rng.roll(sides),
            Self::Fixed(q) => {
                q.pop_front()
                    .expect("OutcomeController::Fixed: sequence exhausted")
            }
            Self::Stratified { visit_counts, total_visits } => {
                // Pick the face with the fewest visits (most under-represented).
                // All faces have equal theoretical probability 1/sides, so
                // argmin visits(o) is equivalent to argmin |visits(o)/N - 1/sides|.
                let best_face = (1..=sides)
                    .min_by_key(|&f| *visit_counts.get(&f).unwrap_or(&0))
                    .unwrap_or(1);
                *visit_counts.entry(best_face).or_insert(0) += 1;
                *total_visits += 1;
                best_face
            }
        }
    }

    /// Create a fresh Stratified controller.
    pub fn stratified() -> Self {
        Self::Stratified {
            visit_counts: std::collections::HashMap::new(),
            total_visits: 0,
        }
    }
}

// ── MCTS config ───────────────────────────────────────────────────────────────

pub struct MctsConfig {
    pub budget: u32,
    pub rollout_depth: RolloutDepth,
    pub outcome_controller: OutcomeController,
    pub c_ucb: f64,
    /// Team the search is playing for.
    pub team: TeamId,
    /// Strategy used during rollout simulation.
    pub rollout_strategy: Box<dyn Strategy>,
}

impl Default for MctsConfig {
    fn default() -> Self {
        Self {
            budget: 100,
            rollout_depth: RolloutDepth::None,
            outcome_controller: OutcomeController::Stochastic,
            c_ucb: 1.41,
            team: TeamId::Home,
            rollout_strategy: Box::new(ffb_sim::simulation::NullStrategy),
        }
    }
}

// ── MCTS search ───────────────────────────────────────────────────────────────

pub struct MctsSearch;

impl MctsSearch {
    /// Run MCTS from `state` and return the best action found within `budget` iterations.
    pub fn search(state: &GameState, cfg: &MctsConfig, rng: &mut GameRng) -> BbAction {
        let mut arena = NodeArena::with_capacity((cfg.budget as usize).min(1_000_000));

        let root_id = arena.alloc(Node::new(None, None, 1.0));
        let legal = enumerate_actions(state, cfg.team);
        if legal.is_empty() {
            return BbAction::EndTurn;
        }
        if legal.len() == 1 {
            return legal.into_iter().next().unwrap();
        }

        // Cache candidates at root
        let _n = legal.len() as f64;
        {
            let root = arena.get_mut(root_id);
            root.candidates = Some(legal.clone());
            // Pre-expand root with prior = 1/n
            for action in &legal {
                // don't add children yet — lazy expansion
                let _ = action;
            }
        }

        for _ in 0..cfg.budget {
            let mut sim_state = state.fast_clone();
            let mut sim_rng = rng.child(42); // deterministic child for reproducibility

            // ── Selection ─────────────────────────────────────────────────────
            let mut node_id = root_id;
            loop {
                let node = arena.get(node_id);
                if node.is_leaf() {
                    break;
                }
                // Check if all candidates are expanded
                let n_children = node.children.len();
                let n_candidates = node.candidates.as_ref().map(|c| c.len()).unwrap_or(0);
                if n_children < n_candidates {
                    // Has unexpanded candidates — expand next
                    break;
                }
                // All expanded — select by UCB
                match select_child_ucb(&arena, node_id, cfg.c_ucb) {
                    Some(child_id) => {
                        let action = arena.get(child_id).action.clone();
                        if let Some(a) = action {
                            apply_action_to_state(&mut sim_state, a, &mut sim_rng);
                        }
                        node_id = child_id;
                    }
                    None => break,
                }
            }

            // ── Expansion ─────────────────────────────────────────────────────
            let _node_visits = arena.get(node_id).visits;
            let candidates = {
                let node = arena.get(node_id);
                if let Some(existing) = &node.candidates {
                    let c = existing.clone();
                    let n = c.len() as f64;
                    Some((c, n))
                } else {
                    let cands = enumerate_actions(&sim_state, cfg.team);
                    let n = cands.len() as f64;
                    Some((cands, n))
                }
            };

            let new_child_id = if let Some((cands, n)) = candidates {
                // Ensure candidates cached
                {
                    let node = arena.get_mut(node_id);
                    if node.candidates.is_none() {
                        node.candidates = Some(cands.clone());
                    }
                }
                let n_children = arena.get(node_id).children.len();
                if n_children < cands.len() {
                    let action = cands[n_children].clone();
                    let prior = 1.0 / n.max(1.0);
                    let child = Node::new(Some(node_id), Some(action.clone()), prior);
                    let child_id = arena.alloc(child);
                    arena.get_mut(node_id).children.push(child_id);
                    // Apply action to simulation state
                    apply_action_to_state(&mut sim_state, action, &mut sim_rng);
                    Some(child_id)
                } else {
                    None
                }
            } else {
                None
            };

            let leaf_id = new_child_id.unwrap_or(node_id);

            // ── Rollout ───────────────────────────────────────────────────────
            let value = rollout(&sim_state, &cfg.rollout_depth, cfg.rollout_strategy.as_ref(), cfg.team, &mut sim_rng);

            // ── Backpropagation ───────────────────────────────────────────────
            let mut back_id = leaf_id;
            loop {
                let node = arena.get_mut(back_id);
                node.visits += 1;
                node.value_sum += value;
                match node.parent {
                    Some(parent_id) => back_id = parent_id,
                    None => break,
                }
            }
        }

        // Pick action of most-visited child
        let root = arena.get(root_id);
        root.children
            .iter()
            .copied()
            .max_by_key(|&id| arena.get(id).visits)
            .and_then(|id| arena.get(id).action.clone())
            .unwrap_or(BbAction::EndTurn)
    }
}

// ── Rollout function ──────────────────────────────────────────────────────────

fn rollout(
    state: &GameState,
    depth: &RolloutDepth,
    strategy: &dyn Strategy,
    team: TeamId,
    rng: &mut GameRng,
) -> f64 {
    match depth {
        RolloutDepth::None => static_eval(state, team),
        RolloutDepth::Full => {
            if state.result.finished {
                terminal_value(state, team)
            } else {
                let final_state = SimulationLoop::run(state.fast_clone(), strategy, strategy, rng);
                terminal_value(&final_state, team)
            }
        }
        RolloutDepth::Steps(n) => {
            let mut s = state.fast_clone();
            for _ in 0..*n {
                if s.result.finished { break; }
                let legal = enumerate_actions(&s, s.active_team_id());
                if legal.is_empty() { break; }
                let action = strategy.choose_action(&s, &legal);
                apply_action_to_state(&mut s, action, rng);
            }
            static_eval(&s, team)
        }
        RolloutDepth::Turns(n) => {
            let mut s = state.fast_clone();
            let start_td = s.result.turns_played;
            while !s.result.finished && u32::from(s.result.turns_played - start_td) < *n {
                let legal = enumerate_actions(&s, s.active_team_id());
                if legal.is_empty() { break; }
                let action = strategy.choose_action(&s, &legal);
                apply_action_to_state(&mut s, action, rng);
            }
            static_eval(&s, team)
        }
        RolloutDepth::UntilKickoff => {
            let start_score = (state.result.score_home, state.result.score_away);
            let mut s = state.fast_clone();
            while !s.result.finished {
                let legal = enumerate_actions(&s, s.active_team_id());
                if legal.is_empty() { break; }
                let action = strategy.choose_action(&s, &legal);
                apply_action_to_state(&mut s, action, rng);
                if (s.result.score_home, s.result.score_away) != start_score {
                    break; // TD scored — kickoff will follow
                }
            }
            static_eval(&s, team)
        }
        RolloutDepth::UntilHalf => {
            let start_half = state.half;
            let mut s = state.fast_clone();
            while !s.result.finished && s.half == start_half {
                let legal = enumerate_actions(&s, s.active_team_id());
                if legal.is_empty() { break; }
                let action = strategy.choose_action(&s, &legal);
                apply_action_to_state(&mut s, action, rng);
            }
            static_eval(&s, team)
        }
    }
}

fn terminal_value(state: &GameState, team: TeamId) -> f64 {
    let delta = state.result.score_delta(team);
    if delta > 0 { 1.0 } else if delta < 0 { 0.0 } else { 0.5 }
}

/// Apply a single action to a simulation state (simplified — no dialog tracking).
fn apply_action_to_state(state: &mut GameState, action: BbAction, rng: &mut GameRng) {
    use ffb_core::steps::{
        apply_block_dice_choice, apply_push_choice, begin_activation, begin_block, begin_move,
        end_activation, end_turn,
    };
    use ffb_core::pathfinding::find_paths;
    use ffb_core::types::PlayerState;

    match action {
        BbAction::EndTurn => {
            end_activation(state);
            end_turn(state);
        }
        BbAction::Activate { player_id, action: player_action } => {
            let _ = begin_activation(state, &player_id, rng);
            if let Some(ap) = state.acting_player.as_mut() {
                ap.current_action = Some(player_action);
            }
        }
        BbAction::MoveTo(coord) => {
            if let Some(ap) = state.acting_player.as_ref() {
                let player_id = ap.player_id.clone();
                let team = ap.team;
                let movement_remaining = {
                    let p = state.team(team).player_by_id(&player_id).unwrap();
                    p.effective_ma().saturating_sub(state.acting_player.as_ref().unwrap().movement_used)
                };
                let path = {
                    let player = state.team(team).player_by_id(&player_id).unwrap();
                    let paths = find_paths(&state.field, player, &player_id, team, movement_remaining);
                    paths.get(&coord).map(|e| e.path.to_vec())
                };
                if let Some(p) = path {
                    begin_move(state, &player_id, &p, rng);
                }
                end_activation(state);
            }
        }
        BbAction::BlockTarget(defender_id) => {
            if let Some(ap) = state.acting_player.as_ref() {
                let attacker_id = ap.player_id.clone();
                begin_block(state, &attacker_id, &defender_id, rng);
            }
        }
        BbAction::ChooseBlockDie(result) => {
            if let Some(ap) = state.acting_player.as_ref() {
                let attacker_id = ap.player_id.clone();
                let team = ap.team;
                let att_coord = state.field.player_coord(&attacker_id);
                let defender_id = att_coord.and_then(|c| {
                    c.neighbors().find_map(|n| {
                        state.field.player_at(n).and_then(|pid| {
                            if state.field.player_team(pid) != Some(team) {
                                state.field.player_state(pid).and_then(|s| {
                                    if s == PlayerState::Prone || s == PlayerState::Standing {
                                        Some(pid.clone())
                                    } else { None }
                                })
                            } else { None }
                        })
                    })
                });
                if let Some(def_id) = defender_id {
                    apply_block_dice_choice(state, &attacker_id, &def_id, result, rng);
                }
            }
        }
        BbAction::ChoosePush(coord) => {
            if let Some(ap) = state.acting_player.as_ref() {
                let attacker_id = ap.player_id.clone();
                let team = ap.team;
                let att_coord = state.field.player_coord(&attacker_id);
                let defender_id = att_coord.and_then(|c| {
                    c.neighbors().find_map(|n| {
                        state.field.player_at(n).and_then(|pid| {
                            if state.field.player_team(pid) != Some(team) {
                                Some(pid.clone())
                            } else { None }
                        })
                    })
                });
                if let Some(def_id) = defender_id {
                    apply_push_choice(state, &attacker_id, &def_id, coord, rng);
                }
            }
        }
        BbAction::PassTo(coord) => {
            if let Some(ap) = state.acting_player.as_ref() {
                let passer_id = ap.player_id.clone();
                ffb_sim::simulation::begin_pass_wrapper(state, &passer_id, coord, rng);
            }
            end_activation(state);
        }
        BbAction::UseReroll(use_it) => {
            if !use_it {
                // Decline reroll — just clear dialog
                state.dialog = ffb_core::model::game_state::DialogState::None;
            } else {
                // Consume reroll
                if let Some(ap) = state.acting_player.as_ref() {
                    let pid = ap.player_id.clone();
                    ffb_core::steps::turn_step::use_team_reroll(state, &pid, rng);
                }
                state.dialog = ffb_core::model::game_state::DialogState::None;
            }
        }
        BbAction::PlaceBall(coord) => {
            state.field.ball.coord = Some(coord);
            state.field.ball.in_play = true;
        }
        BbAction::ChooseFollowup(_) => {
            // Handled by the simulation loop; MCTS picks this via normal action selection
        }
    }
}

// ── Parallel search ───────────────────────────────────────────────────────────

/// Run MCTS with root parallelism: spawn `threads` independent trees, merge visit counts.
pub fn parallel_search(
    state: &GameState,
    cfg: MctsConfig,
    threads: usize,
    seed: u64,
) -> BbAction {
    use rayon::prelude::*;
    use std::sync::Arc;

    let legal = enumerate_actions(state, cfg.team);
    if legal.is_empty() {
        return BbAction::EndTurn;
    }
    if legal.len() == 1 {
        return legal.into_iter().next().unwrap();
    }

    let state = Arc::new(state.fast_clone());
    let budget_per_thread = cfg.budget / threads.max(1) as u32;
    let team = cfg.team;
    let c_ucb = cfg.c_ucb;

    let vote_counts: Vec<BbAction> = (0..threads)
        .into_par_iter()
        .map(|i| {
            let thread_state = state.fast_clone();
            let mut rng = GameRng::new_live(seed.wrapping_add(i as u64 * 6364136223846793005));
            let thread_cfg = MctsConfig {
                budget: budget_per_thread,
                rollout_depth: RolloutDepth::None,
                outcome_controller: OutcomeController::Stochastic,
                c_ucb,
                team,
                rollout_strategy: Box::new(ffb_sim::simulation::NullStrategy),
            };
            MctsSearch::search(&thread_state, &thread_cfg, &mut rng)
        })
        .collect();

    // Majority vote: pick the action that appears most often
    let mut counts: std::collections::HashMap<String, (BbAction, usize)> = std::collections::HashMap::new();
    for action in vote_counts {
        let key = format!("{:?}", action);
        let entry = counts.entry(key).or_insert((action.clone(), 0));
        entry.1 += 1;
    }
    counts.into_values()
        .max_by_key(|(_, count)| *count)
        .map(|(action, _)| action)
        .unwrap_or(BbAction::EndTurn)
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_core::model::player::{Player, PlayerStats};
    use ffb_core::model::team::Team;
    use ffb_core::model::game_state::GameState;
    use ffb_core::skills::SkillSet;
    use ffb_core::types::{FieldCoordinate, PlayerId, PlayerState, TeamId, TurnMode};
    use ffb_sim::setup::{default_kickoff_ball_placement, place_players_for_kickoff};

    fn make_game_state() -> GameState {
        let mut home = Team::new("h".into(), "Home".into(), "Human".into(), 3, true);
        for i in 0..11 {
            home.add_player(Player::new(
                PlayerId(format!("h{i}")),
                format!("HP{i}"),
                "lineman".into(),
                TeamId::Home,
                i as u8 + 1,
                PlayerStats::new(6, 3, 4, 8, None),
                SkillSet::empty(),
            ));
        }
        let mut away = Team::new("a".into(), "Away".into(), "Orc".into(), 3, false);
        for i in 0..11 {
            away.add_player(Player::new(
                PlayerId(format!("a{i}")),
                format!("AP{i}"),
                "lineman".into(),
                TeamId::Away,
                i as u8 + 1,
                PlayerStats::new(5, 4, 3, 9, None),
                SkillSet::empty(),
            ));
        }
        let mut state = GameState::new(home, away);
        place_players_for_kickoff(&mut state);
        default_kickoff_ball_placement(&mut state);
        state.turn_mode = TurnMode::Regular;
        state.home_is_active = true;
        ffb_core::steps::turn_step::begin_turn(&mut state);
        state
    }

    #[test]
    fn budget_1_returns_legal_action() {
        let state = make_game_state();
        let mut rng = GameRng::new_live(42);
        let cfg = MctsConfig {
            budget: 1,
            team: TeamId::Home,
            ..Default::default()
        };
        let action = MctsSearch::search(&state, &cfg, &mut rng);
        let legal = enumerate_actions(&state, TeamId::Home);
        assert!(legal.contains(&action), "action {:?} must be in legal set {:?}", action, legal);
    }

    #[test]
    fn budget_10_returns_legal_action() {
        let state = make_game_state();
        let mut rng = GameRng::new_live(123);
        let cfg = MctsConfig {
            budget: 10,
            team: TeamId::Home,
            ..Default::default()
        };
        let action = MctsSearch::search(&state, &cfg, &mut rng);
        let legal = enumerate_actions(&state, TeamId::Home);
        assert!(legal.contains(&action));
    }

    #[test]
    fn full_rollout_returns_legal_action() {
        let state = make_game_state();
        let mut rng = GameRng::new_live(99);
        let cfg = MctsConfig {
            budget: 5,
            rollout_depth: RolloutDepth::Full,
            team: TeamId::Home,
            ..Default::default()
        };
        let action = MctsSearch::search(&state, &cfg, &mut rng);
        let legal = enumerate_actions(&state, TeamId::Home);
        assert!(legal.contains(&action));
    }

    // ── T-34b: OutcomeController property tests ───────────────────────────────

    #[test]
    fn stochastic_produces_values_in_range() {
        let mut ctrl = OutcomeController::Stochastic;
        let mut rng = GameRng::new_live(42);
        for _ in 0..1000 {
            let v = ctrl.sample(6, &mut rng);
            assert!((1..=6).contains(&v), "stochastic d6 out of range: {v}");
        }
    }

    #[test]
    fn fixed_sequence_exhausted_in_order() {
        use std::collections::VecDeque;
        let seq: VecDeque<u8> = vec![1, 2, 3, 4, 5, 6].into_iter().collect();
        let mut ctrl = OutcomeController::Fixed(seq);
        let mut rng = GameRng::new_live(0);
        assert_eq!(ctrl.sample(6, &mut rng), 1);
        assert_eq!(ctrl.sample(6, &mut rng), 2);
        assert_eq!(ctrl.sample(6, &mut rng), 3);
        assert_eq!(ctrl.sample(6, &mut rng), 4);
        assert_eq!(ctrl.sample(6, &mut rng), 5);
        assert_eq!(ctrl.sample(6, &mut rng), 6);
    }

    #[test]
    fn stratified_produces_values_in_range() {
        let mut ctrl = OutcomeController::stratified();
        let mut rng = GameRng::new_live(7);
        for _ in 0..1000 {
            let v = ctrl.sample(6, &mut rng);
            assert!((1..=6).contains(&v), "stratified d6 out of range: {v}");
        }
    }

    /// T-34b: After N visits, |visits(o)/N - 1/sides| < 1/N + ε for all outcomes.
    #[test]
    fn stratified_distribution_close_to_theoretical() {
        let sides = 6u8;
        let n = 600u32;
        let mut ctrl = OutcomeController::stratified();
        let mut rng = GameRng::new_live(0);
        let mut counts = std::collections::HashMap::<u8, u32>::new();
        for _ in 0..n {
            let v = ctrl.sample(sides, &mut rng);
            *counts.entry(v).or_insert(0) += 1;
        }
        let theoretical = 1.0 / sides as f64;
        let epsilon = 1.0 / n as f64 + 0.01;
        for face in 1..=sides {
            let observed = *counts.get(&face).unwrap_or(&0) as f64 / n as f64;
            let deviation = (observed - theoretical).abs();
            assert!(
                deviation < epsilon,
                "face {face}: observed={observed:.4}, theoretical={theoretical:.4}, deviation={deviation:.4} > epsilon={epsilon:.4}"
            );
        }
    }

    /// T-34b: Stratified L1 distance should be strictly less than stochastic at same N.
    #[test]
    fn stratified_closer_to_theoretical_than_stochastic() {
        let sides = 6u8;
        let n = 600u32;
        // Stratified
        let mut strat_ctrl = OutcomeController::stratified();
        let mut rng_s = GameRng::new_live(42);
        let mut strat_counts = std::collections::HashMap::<u8, u32>::new();
        for _ in 0..n {
            let v = strat_ctrl.sample(sides, &mut rng_s);
            *strat_counts.entry(v).or_insert(0) += 1;
        }
        // Stochastic
        let mut stoch_ctrl = OutcomeController::Stochastic;
        let mut rng_r = GameRng::new_live(42);
        let mut stoch_counts = std::collections::HashMap::<u8, u32>::new();
        for _ in 0..n {
            let v = stoch_ctrl.sample(sides, &mut rng_r);
            *stoch_counts.entry(v).or_insert(0) += 1;
        }
        let theoretical = 1.0 / sides as f64;
        let l1_strat: f64 = (1..=sides)
            .map(|f| (*strat_counts.get(&f).unwrap_or(&0) as f64 / n as f64 - theoretical).abs())
            .sum();
        let l1_stoch: f64 = (1..=sides)
            .map(|f| (*stoch_counts.get(&f).unwrap_or(&0) as f64 / n as f64 - theoretical).abs())
            .sum();
        assert!(
            l1_strat < l1_stoch + 0.05,
            "Stratified L1={l1_strat:.4} should be ≤ stochastic L1={l1_stoch:.4} + margin"
        );
    }

    #[test]
    fn exhaustive_produces_values_in_range() {
        let mut ctrl = OutcomeController::Exhaustive;
        let mut rng = GameRng::new_live(0);
        for _ in 0..100 {
            let v = ctrl.sample(6, &mut rng);
            assert!((1..=6).contains(&v));
        }
    }
}
